use cpal::StreamInstant;

pub struct PatternManager {

}

impl PatternManager {
    pub fn init(time_recv: tokio::sync::watch::Receiver<Option<StreamInstant>>) -> Self {
        
        PatternManager {}
    }
}