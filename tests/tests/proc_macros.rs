// Copyright 2019-2021 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! Example of using proc macro to generate working client and server.

use std::net::SocketAddr;

use jsonrpsee::{
	http_client::HttpClientBuilder, http_server::HttpServerBuilder, types::Error, ws_client::*,
	ws_server::WsServerBuilder,
};

use serde_json::value::RawValue;

mod rpc_impl {
	use jsonrpsee::{
		proc_macros::rpc,
		types::{async_trait, RpcResult},
		ws_server::SubscriptionSink,
	};

	#[rpc(client, server, namespace = "foo")]
	pub trait Rpc {
		#[method(name = "foo")]
		async fn async_method(&self, param_a: u8, param_b: String) -> RpcResult<u16>;

		#[method(name = "bar")]
		fn sync_method(&self) -> RpcResult<u16>;

		#[subscription(name = "sub", item = String)]
		fn sub(&self) -> RpcResult<()>;

		#[subscription(name = "echo", aliases = ["alias_echo"], item = u32)]
		fn sub_with_params(&self, val: u32) -> RpcResult<()>;

		#[method(name = "params")]
		fn params(&self, a: u8, b: &str) -> RpcResult<String> {
			Ok(format!("Called with: {}, {}", a, b))
		}

		#[method(name = "optional_params")]
		fn optional_params(&self, a: u32, b: Option<u32>, c: Option<u32>) -> RpcResult<String> {
			Ok(format!("Called with: {}, {:?}, {:?}", a, b, c))
		}

		#[method(name = "lifetimes")]
		fn lifetimes(
			&self,
			a: &str,
			b: &'_ str,
			c: std::borrow::Cow<'_, str>,
			d: Option<beef::Cow<'_, str>>,
		) -> RpcResult<String> {
			Ok(format!("Called with: {}, {}, {}, {:?}", a, b, c, d))
		}

		#[method(name = "zero_copy_cow")]
		fn zero_copy_cow(&self, a: std::borrow::Cow<'_, str>, b: beef::Cow<'_, str>) -> RpcResult<String> {
			Ok(format!("Zero copy params: {}, {}", matches!(a, std::borrow::Cow::Borrowed(_)), b.is_borrowed()))
		}

		#[method(name = "blocking_call", blocking)]
		fn blocking_call(&self) -> RpcResult<u32> {
			std::thread::sleep(std::time::Duration::from_millis(50));
			Ok(42)
		}
	}

	#[rpc(client, server, namespace = "chain")]
	pub trait ChainApi<Number, Hash, Header, SignedBlock> {
		/// Get header of a relay chain block.
		#[method(name = "getHeader")]
		fn header(&self, hash: Option<Hash>) -> RpcResult<Option<Header>>;

		/// Get header and body of a relay chain block.
		#[method(name = "getBlock")]
		async fn block(&self, hash: Option<Hash>) -> RpcResult<Option<SignedBlock>>;

		/// Get hash of the n-th block in the canon chain.
		///
		/// By default returns latest block hash.
		#[method(name = "getBlockHash")]
		fn block_hash(&self, hash: Hash) -> RpcResult<Option<Hash>>;

		/// Get hash of the last finalized block in the canon chain.
		#[method(name = "getFinalizedHead")]
		fn finalized_head(&self) -> RpcResult<Hash>;

		/// All head subscription
		#[subscription(name = "subscribeAllHeads", item = Header)]
		fn subscribe_all_heads(&self, hash: Hash) -> RpcResult<()>;
	}

	/// Trait to ensure that the trait bounds are correct.
	#[rpc(client, server, namespace = "generic_call")]
	pub trait OnlyGenericCall<I, R> {
		#[method(name = "getHeader")]
		fn call(&self, input: I) -> RpcResult<R>;
	}

	/// Trait to ensure that the trait bounds are correct.
	#[rpc(client, server, namespace = "generic_sub")]
	pub trait OnlyGenericSubscription<Input, R> {
		/// Get header of a relay chain block.
		#[subscription(name = "sub", item = Vec<R>)]
		fn sub(&self, hash: Input) -> RpcResult<()>;
	}

	/// Trait to ensure that the trait bounds are correct.
	#[rpc(client, server, namespace = "generic_with_where_clause")]
	pub trait GenericWhereClause<I, R>
	where
		I: std::fmt::Debug,
		R: Copy + Clone,
	{
		#[method(name = "getHeader")]
		fn call(&self, input: I) -> RpcResult<R>;
	}

	/// Trait to ensure that the trait bounds are correct.
	#[rpc(client, server, namespace = "generic_with_where_clause")]
	pub trait GenericWhereClauseWithTypeBoundsToo<I: Copy + Clone, R>
	where
		I: std::fmt::Debug,
		R: Copy + Clone,
	{
		#[method(name = "getHeader")]
		fn call(&self, input: I) -> RpcResult<R>;
	}

	pub struct RpcServerImpl;

	#[async_trait]
	impl RpcServer for RpcServerImpl {
		async fn async_method(&self, _param_a: u8, _param_b: String) -> RpcResult<u16> {
			Ok(42u16)
		}

		fn sync_method(&self) -> RpcResult<u16> {
			Ok(10u16)
		}

		fn sub(&self, mut sink: SubscriptionSink) -> RpcResult<()> {
			sink.send(&"Response_A")?;
			sink.send(&"Response_B")?;
			Ok(())
		}

		fn sub_with_params(&self, mut sink: SubscriptionSink, val: u32) -> RpcResult<()> {
			sink.send(&val)?;
			sink.send(&val)?;
			Ok(())
		}
	}

	#[async_trait]
	impl OnlyGenericCallServer<String, String> for RpcServerImpl {
		fn call(&self, _: String) -> RpcResult<String> {
			Ok("hello".to_string())
		}
	}

	#[async_trait]
	impl OnlyGenericSubscriptionServer<String, String> for RpcServerImpl {
		fn sub(&self, mut sink: SubscriptionSink, _: String) -> RpcResult<()> {
			sink.send(&"hello")
		}
	}
}

// Use generated implementations of server and client.
use rpc_impl::{RpcClient, RpcServer, RpcServerImpl};

pub async fn websocket_server() -> SocketAddr {
	let server = WsServerBuilder::default().build("127.0.0.1:0").await.unwrap();
	let addr = server.local_addr().unwrap();

	server.start(RpcServerImpl.into_rpc()).unwrap();

	addr
}

#[tokio::test]
async fn proc_macros_generic_ws_client_api() {
	let server_addr = websocket_server().await;
	let server_url = format!("ws://{}", server_addr);
	let client = WsClientBuilder::default().build(&server_url).await.unwrap();

	assert_eq!(client.async_method(10, "a".into()).await.unwrap(), 42);
	assert_eq!(client.sync_method().await.unwrap(), 10);

	// Sub without params
	let mut sub = client.sub().await.unwrap();
	let first_recv = sub.next().await.unwrap();
	assert_eq!(first_recv, Some("Response_A".to_string()));
	let second_recv = sub.next().await.unwrap();
	assert_eq!(second_recv, Some("Response_B".to_string()));

	// Sub with params
	let mut sub = client.sub_with_params(42).await.unwrap();
	let first_recv = sub.next().await.unwrap();
	assert_eq!(first_recv, Some(42));
	let second_recv = sub.next().await.unwrap();
	assert_eq!(second_recv, Some(42));
}

#[tokio::test]
async fn macro_param_parsing() {
	let module = RpcServerImpl.into_rpc();

	let params = RawValue::from_string(r#"[42, "Hello"]"#.into()).ok();
	let result = module.call("foo_params", params).await.unwrap();

	assert_eq!(result, r#"{"jsonrpc":"2.0","result":"Called with: 42, Hello","id":0}"#);
}

#[tokio::test]
async fn macro_optional_param_parsing() {
	let module = RpcServerImpl.into_rpc();

	// Optional param omitted at tail
	let params = RawValue::from_string(r#"[42, 70]"#.into()).ok();
	let result = module.call("foo_optional_params", params).await.unwrap();

	assert_eq!(result, r#"{"jsonrpc":"2.0","result":"Called with: 42, Some(70), None","id":0}"#);

	// Optional param using `null`
	let params = RawValue::from_string(r#"[42, null, 70]"#.into()).ok();
	let result = module.call("foo_optional_params", params).await.unwrap();

	assert_eq!(result, r#"{"jsonrpc":"2.0","result":"Called with: 42, None, Some(70)","id":0}"#);

	// Named params using a map
	let params = RawValue::from_string(r#"{"a": 22, "c": 50}"#.into()).ok();
	let result = module.call("foo_optional_params", params).await.unwrap();
	assert_eq!(result, r#"{"jsonrpc":"2.0","result":"Called with: 22, None, Some(50)","id":0}"#);
}

#[tokio::test]
async fn macro_lifetimes_parsing() {
	let module = RpcServerImpl.into_rpc();

	let params = RawValue::from_string(r#"["foo", "bar", "baz", "qux"]"#.into()).ok();
	let result = module.call("foo_lifetimes", params).await.unwrap();

	assert_eq!(result, r#"{"jsonrpc":"2.0","result":"Called with: foo, bar, baz, Some(\"qux\")","id":0}"#);
}

#[tokio::test]
async fn macro_zero_copy_cow() {
	let module = RpcServerImpl.into_rpc();

	let params = RawValue::from_string(r#"["foo", "bar"]"#.into()).ok();
	let result = module.call("foo_zero_copy_cow", params).await.unwrap();

	// std::borrow::Cow<str> always deserialized to owned variant here
	assert_eq!(result, r#"{"jsonrpc":"2.0","result":"Zero copy params: false, true","id":0}"#);

	// serde_json will have to allocate a new string to replace `\t` with byte 0x09 (tab)
	let params = RawValue::from_string(r#"["\tfoo", "\tbar"]"#.into()).ok();
	let result = module.call("foo_zero_copy_cow", params).await.unwrap();

	assert_eq!(result, r#"{"jsonrpc":"2.0","result":"Zero copy params: false, false","id":0}"#);
}

// Disabled on MacOS as GH CI timings on Mac vary wildly (~100ms) making this test fail.
#[cfg(not(target_os = "macos"))]
#[tokio::test]
async fn multiple_blocking_calls_overlap() {
	use std::time::{Duration, Instant};

	let module = RpcServerImpl.into_rpc();

	let params = RawValue::from_string("[]".into()).ok();

	let futures = std::iter::repeat_with(|| module.call("foo_blocking_call", params.clone())).take(4);
	let now = Instant::now();
	let results = futures::future::join_all(futures).await;
	let elapsed = now.elapsed();

	for result in results {
		let result = serde_json::from_str::<serde_json::Value>(&result.unwrap()).unwrap();
		assert_eq!(result["result"], 42);
	}

	// Each request takes 50ms, added 10ms margin for scheduling
	assert!(elapsed < Duration::from_millis(60), "Expected less than 60ms, got {:?}", elapsed);
}

#[tokio::test]
async fn subscriptions_do_not_work_for_http_servers() {
	let htserver = HttpServerBuilder::default().build("127.0.0.1:0").unwrap();
	let addr = htserver.local_addr().unwrap();
	let htserver_url = format!("http://{}", addr);
	let _handle = htserver.start(RpcServerImpl.into_rpc()).unwrap();

	let htclient = HttpClientBuilder::default().build(&htserver_url).unwrap();

	assert_eq!(htclient.sync_method().await.unwrap(), 10);
	assert!(htclient.sub().await.is_err());
	assert!(matches!(htclient.sub().await, Err(Error::HttpNotImplemented)));
	assert_eq!(htclient.sync_method().await.unwrap(), 10);
}
