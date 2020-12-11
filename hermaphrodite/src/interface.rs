//use crate::server::Player;

pub trait MinecraftServer<'l>: Send + Sync {
	/// Retrieves the message of the day.
	fn message_of_the_day(&self) -> String;

	fn event_listener_register<E>(&self, listener: &'l dyn Fn(&E, &Self))
		where Self: Sized, E: Event + 'static;

	fn event_dispatch<E>(&self, event: E)
		where Self: Sized, E: Event + 'static;

	fn new_pov(&self, name: Box<str>);
}

pub trait ChunkFetcher {
	
}

pub trait Event {
	type Intent;

	fn push_intent(&self, intent: Self::Intent);

	fn handle<'l, S>(self, server: &S)
		where Self: Sized, S: MinecraftServer<'l>;
}
