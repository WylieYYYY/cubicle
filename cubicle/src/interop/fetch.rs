use std::io::{self, ErrorKind};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use async_std::io::prelude::*;
use async_std::sync::Mutex;
use derivative::Derivative;
use js_sys::{Error, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    ReadableStream, ReadableStreamByobReader,
    ReadableStreamGetReaderOptions, ReadableStreamReaderMode,

    Request, RequestInit, RequestMode, Response
};

use super::bits;
use crate::interop;
use crate::util::errors::CustomError;

#[derive(Copy, Clone, Eq, PartialEq)]
enum FetchState { Delivered, Consumed, Done }

pub struct Fetch {
    reader: ReadableStreamByobReader,
    resolve_read_then: Closure<dyn FnMut(JsValue)>,
    reject_read_then: Closure<dyn FnMut(JsValue)>,
    state: Arc<Mutex<SharedState>>
}

#[derive(Derivative)]
#[derivative(Default)]
struct SharedState {
    buffer: Uint8Array,
    waker: Option<Waker>,
    #[derivative(Default(value="Some(Ok(FetchState::Consumed))"))]
    success: Option<io::Result<FetchState>>
}

impl Fetch {
    pub async fn get_stream(url: &str) -> Result<Self, CustomError> {
        Self::try_from(Response::from(get(url).await?).body()
            .ok_or(CustomError::FailedFetchRequest {
                message: String::from("response has no body")
            })?)
    }

    fn read_to_buffer(self: Pin<&mut Self>, cx: &mut Context<'_>, size: usize)
    -> Poll<io::Result<FetchState>> {
        let Some(mut state) = self.state.try_lock() else {
            return Poll::Pending;
        };

        match state.success.take() {
            None => return Poll::Pending,
            Some(Ok(FetchState::Consumed)) => (),
            Some(done) => {
                state.success = Some(Ok(*done.as_ref()
                    .unwrap_or(&FetchState::Consumed)));
                return Poll::Ready(done);
            }
        }

        let size = u32::try_from(size).unwrap_or(u32::MAX);
        if size != state.buffer.length() {
            state.buffer = Uint8Array::new_with_length(size);
        }

        state.waker = Some(cx.waker().clone());
        drop(self.reader.read_with_array_buffer_view(&state.buffer)
            .then2(&self.resolve_read_then, &self.reject_read_then));
        Poll::Pending
    }

    fn read_thens(state: Arc<Mutex<SharedState>>, resolve: bool)
    -> Closure<dyn FnMut(JsValue)> {
        Closure::new(move |value: JsValue| {
            use FetchState::*;

            let mut state = state.try_lock()
                .expect("promise chaining should be executed synchronously");
            if resolve {
                let done = interop::get_or_standard_mismatch(&value, "done")
                    .and_then(interop::cast_or_standard_mismatch)
                    .and_then(|done| Ok(if done { Done } else { Delivered }))
                    .or(Err(io::Error::new(ErrorKind::InvalidData, 
                        "browser's did not return a valid done value")));
                state.success = Some(done);
                state.buffer = bits::reader_value_done_pair::buffer(&value);
            } else {
                let io_error = io::Error::new(ErrorKind::BrokenPipe,
                    Error::from(value).message().as_string()
                    .expect("cast of javascript string always succeed"));
                state.success = Some(Err(io_error));
            }
            if let Some(waker) = &state.waker { waker.clone().wake() }
        })
    }
}

impl Read for Fetch {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>,
        buf: &mut [u8]) -> Poll<io::Result<usize>> {
        let ret = self.as_mut().read_to_buffer(cx, buf.len());
        if let Poll::Ready(Ok(done)) = ret {
            let mut state = self.state.try_lock()
                .expect("mutex held by promises should be unlocked");

            if done == FetchState::Done { return Poll::Ready(Ok(0)); }

            let read_length = state.buffer.length() as usize;
            state.buffer.copy_to(&mut buf[..read_length]);
            state.success = Some(Ok(FetchState::Consumed));
            Poll::Ready(Ok(read_length))
        } else { ret.map_ok(|_| unreachable!("all ok results have branched")) }
    }
}

impl TryFrom<ReadableStream> for Fetch {
    type Error = CustomError;
    fn try_from(value: ReadableStream) -> Result<Self, Self::Error> {
        let mut reader_options = ReadableStreamGetReaderOptions::new();
        reader_options.mode(ReadableStreamReaderMode::Byob);
        let reader = value.get_reader_with_options(&reader_options)
            .dyn_into().or(Err(CustomError::StandardMismatch {
                message: String::from("a BYOB reader is expected")
            }))?;
        let state = Arc::new(Mutex::new(SharedState::default()));
        Ok(Self {
            reader, resolve_read_then: Self::read_thens(state.clone(), true),
            reject_read_then: Self::read_thens(state.clone(), false), state
        })
    }
}

pub async fn get(url: &str) -> Result<Response, CustomError> {
    let mut connection_options = RequestInit::new();
    connection_options.method("GET").mode(RequestMode::Cors);
    let request = Request::new_with_str_and_init(url, &connection_options)
        .or(Err(CustomError::FailedFetchRequest {
            message: String::from("credentials in URL not supported")
        }))?;
    let window = web_sys::window().ok_or(CustomError::StandardMismatch {
        message: String::from("window should exist in page")
    })?;
    let resp = JsFuture::from(window.fetch_with_request(&request)).await
        .or(Err(CustomError::FailedFetchRequest {
            message: String::from("network error")
        }))?;
    Ok(Response::from(resp))
}
