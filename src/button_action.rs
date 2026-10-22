/// Only registered actions or individually audited exceptions may activate controls.
pub(crate) trait ButtonAction<Message> {
    fn into_message(self) -> Message;
}

pub(crate) fn message<Message>(action: impl ButtonAction<Message>) -> Message {
    action.into_message()
}
