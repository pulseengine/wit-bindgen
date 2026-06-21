//@ wasmtime-flags = '-Wcomponent-model-async'

// Exercises the single-item write path `StreamWriter::write_one`, which reuses
// an internal cached buffer to avoid a per-item heap allocation. Writing the
// same writer repeatedly is what reuses (and therefore must correctly reset)
// the cached buffer between calls.

include!(env!("BINDINGS"));

use crate::my::test::i::*;

struct Component;

export!(Component);

impl Guest for Component {
    async fn run() {
        let (mut tx, rx) = wit_stream::new();
        let test = async {
            // Multiple single-item writes through the same writer so the cached
            // buffer is reused across calls. A successful send returns `None`.
            assert!(tx.write_one(10).await.is_none());
            assert!(tx.write_one(20).await.is_none());
            assert!(tx.write_one(30).await.is_none());

            // Drop the writer so the reader observes end-of-stream.
            drop(tx);
        };
        let ((), ()) = futures::join!(test, read_stream(rx));
    }
}
