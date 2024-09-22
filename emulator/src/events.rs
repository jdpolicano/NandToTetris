/// the types of message events between the CPU thread and the main thread
pub enum CpuThreadMessage {
    /// the CPU thread has finished executing
    Finished,
    /// something went wrong with the CPU thread
    Error,
    /// a heartbeat message to keep the CPU thread alive
    Heartbeat,
}

/// The types of events that can be sent by the main thread
pub enum MainThreadMessage {
    /// the main thread has finished executing
    Finished,
    /// the main thread has encountered an error
    Error,
    /// a message telling the cpu to start
    CpuStart,
}
