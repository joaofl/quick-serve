use tracing::{Subscriber, Event};
use tokio::sync::broadcast;


pub struct MySubscriber {
    subscriber: tracing_subscriber::fmt::Subscriber,
    pub sender: tokio::sync::broadcast::Sender<String>,
}

impl MySubscriber {
    pub fn new() -> Self {
        // let subscriber_builder = tracing_subscriber::fmt()
        // // Configure formatting settings
        // .with_target(false)
        // .with_level(true)
        // .with_timer(tracing_subscriber::fmt::time::uptime())
        // .with_max_level(Level::DEBUG)
        // .compact(); 
        // // Set the subscriber as the default.
        // // .init();

        // let x = subscriber_builder.set_default();
        let subscriber = tracing_subscriber::fmt::Subscriber::new();
        let (sender, _) = broadcast::channel(1);

        // tracing::subscriber::set_default(subscriber).expect("Unable to set the default");

        Self {
            subscriber,
            sender,
        }
    }
}

impl Subscriber for MySubscriber {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        self.subscriber.enabled(metadata)
    }

    fn new_span(&self, span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        self.subscriber.new_span(span)
    }

    fn record(&self, span: &tracing::span::Id, values: &tracing::span::Record<'_>) {
        self.subscriber.record(span, values)
    }

    fn record_follows_from(&self, span: &tracing::span::Id, follows: &tracing::span::Id) {
        self.subscriber.record_follows_from(span, follows)
    }

    fn event(&self, event: &Event<'_>) {
        // Send log text in data channel
        // let _ = self.sender.send(event.metadata().name().to_string());
        let e = format!("{:?}", event);
        let _ = self.sender.send(e);


        self.subscriber.event(event)
    }

    fn enter(&self, span: &tracing::span::Id) {
        self.subscriber.enter(span)
    }

    fn exit(&self, span: &tracing::span::Id) {
        self.subscriber.exit(span)
    }
}