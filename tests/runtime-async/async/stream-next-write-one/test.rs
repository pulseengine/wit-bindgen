use wit_bindgen::StreamReader;

include!(env!("BINDINGS"));

struct Component;

export!(Component);

impl crate::exports::my::test::i::Guest for Component {
    async fn read_stream(mut x: StreamReader<u8>) {
        // Exercises the single-item read path `StreamReader::next`, which reuses
        // an internal cached buffer across calls. Reading repeatedly from the
        // same reader is what reuses (and therefore must correctly reset) the
        // cached buffer between calls.
        assert_eq!(x.next().await, Some(10));
        assert_eq!(x.next().await, Some(20));
        assert_eq!(x.next().await, Some(30));

        // After the writer is dropped the stream is exhausted.
        assert_eq!(x.next().await, None);
    }
}
