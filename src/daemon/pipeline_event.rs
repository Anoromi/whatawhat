pub enum PipeEvent<T> {
    Next(T),
    Close,
}
