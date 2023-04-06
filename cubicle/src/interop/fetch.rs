use std::io::{self, ErrorKind};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use async_std::io::prelude::*;
use js_sys::{ArrayBuffer, Error, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    ReadableStream, ReadableStreamByobReader,
    ReadableStreamGetReaderOptions, ReadableStreamReaderMode,

    Request, RequestInit, RequestMode, Response
};

use super::bits;
use crate::util::{self, errors::BrowserApiError};

pub struct FetchReader {
    reader: ReadableStreamByobReader,
    resolve_read_then: Closure<dyn FnMut(JsValue)>,
    reject_read_then: Closure<dyn FnMut(JsValue)>,
    read_finally: Closure<dyn FnMut()>,
    state: Arc<Mutex<SharedState>>
}

struct SharedState {
    buffer: Option<Uint8Array>,
    waker: Option<Waker>,
    success: Option<io::Result<()>>
}

impl FetchReader {
    fn read_to_buffer(self: Pin<&mut Self>, cx: &mut Context<'_>,
        length: usize) -> Poll<io::Result<()>> {
        let length = util::usize_to_u32(length);
        let mut state = match self.state.lock() {
            Err(_) => return Poll::Pending,
            Ok(state) => state
        };

        if let Some(done_state) = state.success.take() {
            return Poll::Ready(done_state);
        }

        state.waker = Some(cx.waker().clone());
        if state.buffer.is_some() { return Poll::Pending; }
        drop(self.reader.read_with_array_buffer_view(
            state.buffer.insert(Uint8Array::new(&ArrayBuffer::new(length))))
            .then2(&self.resolve_read_then, &self.reject_read_then)
            .finally(&self.read_finally)
        );
        Poll::Pending
    }

    fn read_thens(shared_state: Arc<Mutex<SharedState>>, resolve: bool)
        -> Closure<dyn FnMut(JsValue)> {
        Closure::new(move |value: JsValue| {
            let mut shared_state = shared_state.lock()
                .expect("promise chaining should be executed synchronously");
            shared_state.success = Some(Ok(()));
            if resolve {
                shared_state.buffer = Some(
                    bits::reader_value_done_pair::buffer(&value));
            } else {
                let io_error = io::Error::new(ErrorKind::BrokenPipe,
                    Error::from(value).message().as_string()
                    .expect("cast of javascript string always succeed"));
                shared_state.success = Some(Err(io_error));
            }
        })
    }
}

impl Read for FetchReader {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>,
        buf: &mut [u8]) -> Poll<io::Result<usize>> {
        let ret = self.as_mut().read_to_buffer(cx, buf.len());
        if let Poll::Ready(Ok(())) = ret {
            let state = self.state.lock()
                .expect("mutex held by promises should be unlocked");
            state.buffer.clone()
                .expect("buffer must be assigned for ready state")
                .copy_to(buf);
        }
        ret.map_ok(|_| buf.len())
    }
}

impl TryFrom<ReadableStream> for FetchReader {
    type Error = JsValue;
    fn try_from(value: ReadableStream) -> Result<Self, Self::Error> {
        let mut reader_options = ReadableStreamGetReaderOptions::new();
        reader_options.mode(ReadableStreamReaderMode::Byob);
        let reader = value.get_reader_with_options(&reader_options)
            .dyn_into().or(Err(JsError::from(BrowserApiError::StandardMismatch {
                message: String::from("a BYOB reader is expected")
            })))?;
        let state = Arc::new(Mutex::new(SharedState::default()));
        let finally_state = state.clone();
        let read_finally = Closure::new(Box::new(move || {
            let shared_state = finally_state.lock()
                .expect("promise chaining should be executed synchronously");
            if let Some(waker) = shared_state.waker.as_ref() {
                waker.clone().wake();
            }
        }));
        Ok(Self {
            reader,
            resolve_read_then: Self::read_thens(state.clone(), true),
            reject_read_then: Self::read_thens(state.clone(), false),
            read_finally, state
        })
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self { buffer: None, waker: None, success: None }
    }
}

pub async fn get(url: &str) -> Result<Response, JsValue> {
    let mut connection_options = RequestInit::new();
    connection_options.method("GET").mode(RequestMode::Cors);
    let request = Request::new_with_str_and_init(url, &connection_options)?;
    let window = web_sys::window().ok_or(JsError::from(
        BrowserApiError::StandardMismatch {
            message: String::from("window should exist in page")
        }))?;
    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    Ok(Response::from(resp))
}
