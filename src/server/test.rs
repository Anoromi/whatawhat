use std::marker::PhantomData;

struct JSONParser {}

impl Parser for JSONParser {
    fn deserialize() {}
}

trait Parser {
    fn deserialize();
}

struct Deserializer<P: Parser> {
    phantom: PhantomData<P>,
}

impl<P: Parser> Deserializer<P> {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn hello(&self) {
        P::deserialize();
    }
}

fn hello() {
    let k: Deserializer<JSONParser> = Deserializer::new();
}
