use crate::*;

use riddle_common::eventpub::{EventPub, EventSub};
use riddle_platform_common::{LogicalPosition, PlatformEvent, WindowId};

use std::{collections::HashMap, sync::Mutex};

struct WindowInputState {
	mouse: MouseState,
	keyboard: KeyboardState,
}

/// The Riddle input system core state, along with [`InputMainThreadState`].
///
/// This stores the thread safe input state which can be queried to inspect the
/// status of input devices. It is updated by [`InputMainThreadState::process_input`].
#[derive(Clone)]
pub struct InputSystem {
	internal: std::sync::Arc<InputSystemInternal>,
}

pub struct InputSystemInternal {
	window_states: Mutex<HashMap<WindowId, WindowInputState>>,
	gamepad_states: Mutex<GamePadStateMap>,
	outgoing_input_events: Mutex<Vec<InputEvent>>,
}

impl InputSystem {
	/// Query the cursor position with respect to a given window.
	///
	/// If no mouse position infomation is available for the given window, the
	/// position will be (0,0)
	///
	/// # Example
	///
	/// ```
	/// # use riddle_input::{ext::*, *}; use riddle_common::eventpub::*; use riddle_platform_common::*;
	/// # fn main() -> Result<(), InputError> {
	/// # let platform_events: EventPub<PlatformEvent> = EventPub::new();
	/// # let (input_system, mut main_thread_state) = InputSystem::new_system_pair(&platform_events)?;
	/// # let window = WindowId::new(0);
	/// // The initial mouse position is (0,0)
	/// assert_eq!(LogicalPosition{ x: 0, y: 0}, input_system.mouse_pos(window));
	///
	/// // The platform system emits cursor move events
	/// // [..]
	/// # platform_events.dispatch(PlatformEvent::CursorMove {
	/// #     window: WindowId::new(0),
	/// #     position: LogicalPosition{ x: 10, y: 10}});
	/// # main_thread_state.process_input();
	///
	/// // The reported mouse position has changed
	/// assert_ne!(LogicalPosition{ x: 0, y: 0}, input_system.mouse_pos(window));
	/// # Ok(()) }
	/// ```
	pub fn mouse_pos(&self, window: WindowId) -> LogicalPosition {
		self.with_window_state(window, |w| w.mouse.position())
	}

	/// Check if the specified mouse button is down with respect to a given window.
	///
	/// If there is no state recorded for the mouse for the given window the result will
	/// be false
	///
	/// # Example
	///
	/// ```
	/// # use riddle_input::{ext::*, *}; use riddle_common::eventpub::*; use riddle_platform_common::*;
	/// # fn main() -> Result<(), InputError> {
	/// # let platform_events: EventPub<PlatformEvent> = EventPub::new();
	/// # let (input_system, mut main_thread_state) = InputSystem::new_system_pair(&platform_events)?;
	/// # let window = WindowId::new(0);
	/// // The initial mouse position is (0,0)
	/// assert_eq!(false, input_system.is_mouse_button_down(window, MouseButton::Left));
	///
	/// // The platform system emits cursor move events
	/// // [..]
	/// # platform_events.dispatch(PlatformEvent::MouseButtonDown {
	/// #     window: WindowId::new(0),
	/// #     button: MouseButton::Left});
	/// # main_thread_state.process_input();
	///
	/// // The reported mouse position has changed
	/// assert_eq!(true, input_system.is_mouse_button_down(window, MouseButton::Left));
	/// # Ok(()) }
	/// ```
	pub fn is_mouse_button_down(&self, window: WindowId, button: MouseButton) -> bool {
		self.with_window_state(window, |w| w.mouse.is_button_down(button))
	}

	/// Query the keyboard scancode state with respect to a given window. See
	/// [`Scancode`] for details on what a scancode represents.
	///
	/// If no keyboard infomation is available for the given window, the button
	/// will be considered to not be down.
	///
	/// # Example
	///
	/// ```
	/// # use riddle_input::{ext::*, *}; use riddle_common::eventpub::*; use riddle_platform_common::*;
	/// # fn main() -> Result<(), InputError> {
	/// # let platform_events: EventPub<PlatformEvent> = EventPub::new();
	/// # let (input_system, mut main_thread_state) = InputSystem::new_system_pair(&platform_events)?;
	/// # let window = WindowId::new(0);
	/// // The initial key state is that the button is unpressed
	/// assert_eq!(false, input_system.is_key_down(window, Scancode::Escape));
	///
	/// // The platform system emits key press event
	/// // [..]
	/// # platform_events.dispatch(PlatformEvent::KeyDown {
	/// #     window: WindowId::new(0),
	/// #     vkey: Some(VirtualKey::Escape),
	/// #     scancode: Scancode::Escape,
	/// #     platform_scancode: 0});
	/// # main_thread_state.process_input();
	///
	/// // The reported key state has changed
	/// assert_eq!(true, input_system.is_key_down(window, Scancode::Escape));
	/// # Ok(()) }
	/// ```
	pub fn is_key_down(&self, window: WindowId, scancode: Scancode) -> bool {
		self.with_window_state(window, |w| w.keyboard.is_key_down(scancode))
	}

	/// Query the keyboard virtual key state with respect to a given window. See
	/// [`VirtualKey`] for details on what a virtual key represents.
	///
	/// If no keyboard infomation is available for the given window, the button
	/// will be considered to not be down.
	///
	/// # Example
	///
	/// ```
	/// # use riddle_input::{ext::*, *}; use riddle_common::eventpub::*; use riddle_platform_common::*;
	/// # fn main() -> Result<(), InputError> {
	/// # let platform_events: EventPub<PlatformEvent> = EventPub::new();
	/// # let (input_system, mut main_thread_state) = InputSystem::new_system_pair(&platform_events)?;
	/// # let window = WindowId::new(0);
	/// // The initial key state is that the button is unpressed
	/// assert_eq!(false, input_system.is_vkey_down(window, VirtualKey::Escape));
	///
	/// // The platform system emits key press event
	/// // [..]
	/// # platform_events.dispatch(PlatformEvent::KeyDown {
	/// #     window: WindowId::new(0),
	/// #     vkey: Some(VirtualKey::Escape),
	/// #     scancode: Scancode::Escape,
	/// #     platform_scancode: 0});
	/// # main_thread_state.process_input();
	///
	/// // The reported key state has changed
	/// assert_eq!(true, input_system.is_vkey_down(window, VirtualKey::Escape));
	/// # Ok(()) }
	/// ```
	pub fn is_vkey_down(&self, window: WindowId, vkey: VirtualKey) -> bool {
		self.with_window_state(window, |w| w.keyboard.is_vkey_down(vkey))
	}

	/// The current state of keyboard modifiers with respect to a given window.
	///
	/// If no state has been set all modifiers are considered unset.
	///
	/// # Example
	///
	/// ```
	/// # use riddle_input::{ext::*, *}; use riddle_common::eventpub::*; use riddle_platform_common::*;
	/// # fn main() -> Result<(), InputError> {
	/// # let platform_events: EventPub<PlatformEvent> = EventPub::new();
	/// # let (input_system, mut main_thread_state) = InputSystem::new_system_pair(&platform_events)?;
	/// # let window = WindowId::new(0);
	/// // The initial mouse position is (0,0)
	/// assert_eq!(false, input_system.keyboard_modifiers(window).ctrl);
	///
	/// // The platform system emits key down event for Ctrl key
	/// // [..]
	/// # platform_events.dispatch(PlatformEvent::KeyDown {
	/// #     window: WindowId::new(0),
	/// #     vkey: Some(VirtualKey::LeftControl),
	/// #     scancode: Scancode::LeftControl,
	/// #     platform_scancode: 0});
	/// # main_thread_state.process_input();
	///
	/// // The reported mouse position has changed
	/// assert_eq!(true, input_system.keyboard_modifiers(window).ctrl);
	/// # Ok(()) }
	/// ```
	pub fn keyboard_modifiers(&self, window: WindowId) -> KeyboardModifiers {
		self.with_window_state(window, |w| w.keyboard.modifiers())
	}

	/// Get the [`GamePadId`] of the last gamepad which issued any event.
	///
	/// Can be used a very simple way to pick a gamepad to consider the "active"
	/// gamepad. Handling [`InputEvent::GamePadConnected`] and similar events
	/// allows for more fine grained control.
	///
	/// # Example
	///
	/// ```no_run
	/// # use riddle_input::{ext::*, *}; use riddle_common::eventpub::*; use riddle_platform_common::*;
	/// # fn main() -> Result<(), InputError> {
	/// # let platform_events: EventPub<PlatformEvent> = EventPub::new();
	/// # let (input_system, mut main_thread_state) = InputSystem::new_system_pair(&platform_events)?;
	/// # let window = WindowId::new(0);
	/// // The initial gamepad is None
	/// assert_eq!(None, input_system.last_active_gamepad());
	///
	/// // Controller button press events are processed
	/// // [..]
	/// # // Currently no support for faking a controller event
	/// # panic!();
	///
	/// // The reported active controller has changed
	/// assert_ne!(None, input_system.last_active_gamepad());
	/// # Ok(()) }
	/// ```
	pub fn last_active_gamepad(&self) -> Option<GamePadId> {
		self.internal
			.gamepad_states
			.lock()
			.unwrap()
			.last_active_pad()
	}

	/// Check if a specific button is pressed for a given gamepad.
	///
	/// If no state has been set all buttons are considered not to be down.
	///
	/// ```no_run
	/// # use riddle_input::{ext::*, *};
	/// # let input_system: InputSystem = todo!();
	/// # let gamepad: GamePadId = todo!();
	/// // The initial button state is false
	/// assert_eq!(false, input_system.is_gamepad_button_down(gamepad, GamePadButton::North));
	///
	/// // Controller button press events are processed
	/// // [..]
	/// # // Currently no support for faking a controller event
	/// # panic!();
	///
	/// // The reported button state has changed
	/// assert_eq!(true, input_system.is_gamepad_button_down(gamepad, GamePadButton::North));
	/// ```
	pub fn is_gamepad_button_down(&self, gamepad: GamePadId, button: GamePadButton) -> bool {
		self.internal
			.gamepad_states
			.lock()
			.unwrap()
			.is_button_down(gamepad, button)
	}

	/// Get the value of a specific axis for a specific gamepad
	///
	/// If no state has been set all axes are considered to have value `0.0`.
	///
	/// ```no_run
	/// # use riddle_input::{ext::*, *};
	/// # let input_system: InputSystem = todo!();
	/// # let gamepad: GamePadId = todo!();
	/// // The initial button state is false
	/// assert_eq!(0.0, input_system.gamepad_axis_value(gamepad, GamePadAxis::LeftStickX));
	///
	/// // Controller axis events are processed
	/// // [..]
	/// # // Currently no support for faking a controller event
	/// # panic!();
	///
	/// // The reported axis value has changed
	/// assert_ne!(0.0, input_system.gamepad_axis_value(gamepad, GamePadAxis::LeftStickX));
	/// ```
	pub fn gamepad_axis_value(&self, gamepad: GamePadId, axis: GamePadAxis) -> f32 {
		self.internal
			.gamepad_states
			.lock()
			.unwrap()
			.axis_value(gamepad, axis)
	}

	fn with_window_state<R, F>(&self, window: WindowId, f: F) -> R
	where
		F: FnOnce(&WindowInputState) -> R,
	{
		let mut ms = self.internal.window_states.lock().unwrap();
		if !ms.contains_key(&window) {
			ms.insert(window, Default::default());
		}
		f(ms.get(&window).unwrap())
	}

	fn with_window_state_mut<R, F>(&self, window: WindowId, f: F) -> R
	where
		F: FnOnce(&mut WindowInputState) -> R,
	{
		let mut ms = self.internal.window_states.lock().unwrap();
		if !ms.contains_key(&window) {
			ms.insert(window, Default::default());
		}
		f(ms.get_mut(&window).unwrap())
	}

	fn cursor_moved(&self, window: WindowId, logical_position: LogicalPosition) {
		self.with_window_state_mut(window, |w| w.mouse.set_position(logical_position));
		self.send_input_event(InputEvent::CursorMove {
			window,
			position: logical_position,
		});
	}

	fn mouse_down(&self, window: WindowId, button: MouseButton) {
		self.with_window_state_mut(window, |w| w.mouse.button_down(button));
		self.send_input_event(InputEvent::MouseButtonDown { window, button });
	}

	fn mouse_up(&self, window: WindowId, button: MouseButton) {
		self.with_window_state_mut(window, |w| w.mouse.button_up(button));
		self.send_input_event(InputEvent::MouseButtonUp { window, button });
	}

	fn key_down(&self, window: WindowId, scancode: Scancode, vkey: Option<VirtualKey>) {
		let modifiers = self.with_window_state_mut(window, |w| {
			w.keyboard.key_down(scancode, vkey);
			w.keyboard.modifiers()
		});
		self.send_input_event(InputEvent::KeyDown {
			window,
			scancode,
			vkey,
			modifiers,
		});
	}

	fn key_up(&self, window: WindowId, scancode: Scancode, vkey: Option<VirtualKey>) {
		let modifiers = self.with_window_state_mut(window, |w| {
			w.keyboard.key_up(scancode, vkey);
			w.keyboard.modifiers()
		});
		self.send_input_event(InputEvent::KeyUp {
			window,
			scancode,
			vkey,
			modifiers,
		})
	}

	fn text_input(&self, window: WindowId, text: String) {
		self.send_input_event(InputEvent::TextInput { window, text });
	}

	fn event_filter(event: &PlatformEvent) -> bool {
		matches!(
			event,
			PlatformEvent::CursorMove { .. }
				| PlatformEvent::MouseButtonUp { .. }
				| PlatformEvent::MouseButtonDown { .. }
				| PlatformEvent::KeyUp { .. }
				| PlatformEvent::KeyDown { .. }
				| PlatformEvent::TextInput { .. }
		)
	}

	fn send_input_event(&self, event: InputEvent) {
		self.internal
			.outgoing_input_events
			.lock()
			.unwrap()
			.push(event);
	}
}

impl ext::InputSystemExt for InputSystem {
	fn new_system_pair(
		sys_events: &EventPub<PlatformEvent>,
	) -> Result<(InputSystem, InputMainThreadState)> {
		let event_sub = EventSub::new_with_filter(Self::event_filter);
		sys_events.attach(&event_sub);

		let gilrs = gilrs::Gilrs::new().map_err(|_| InputError::Init("Gilrs init failure"))?;

		let internal = InputSystemInternal {
			window_states: Mutex::new(HashMap::new()),
			gamepad_states: Mutex::new(GamePadStateMap::new()),
			outgoing_input_events: Mutex::new(vec![]),
		};
		let system = InputSystem {
			internal: std::sync::Arc::new(internal),
		};

		let main_thread = InputMainThreadState {
			system: system.clone(),
			event_sub,
			gilrs,
		};

		Ok((system, main_thread))
	}

	fn take_input_events(&self) -> Vec<InputEvent> {
		std::mem::take(&mut self.internal.outgoing_input_events.lock().unwrap())
	}
}

impl Default for WindowInputState {
	fn default() -> Self {
		Self {
			mouse: Default::default(),
			keyboard: Default::default(),
		}
	}
}

/// The portion of the input system that needs to remain on a single thread.
///
/// Constructed paired with its thread-safe counterpart via [`ext::InputSystemExt::new_system_pair`].
pub struct InputMainThreadState {
	system: InputSystem,

	event_sub: EventSub<PlatformEvent>,
	gilrs: gilrs::Gilrs,
}

impl InputMainThreadState {
	/// Process all input sources, updating the static view of the known input
	/// devices.
	///
	/// This may produce InputEvents which can be consumed via [`ext::InputSystemExt::take_input_events`].
	pub fn process_input(&mut self) {
		while let Some(gilrs::Event { event, id, .. }) = self.gilrs.next_event() {
			match event {
				gilrs::EventType::ButtonPressed(button, _) => {
					if let Ok(button) = GamePadButton::try_from(button) {
						self.system
							.internal
							.gamepad_states
							.lock()
							.unwrap()
							.button_down(id.into(), button);
						self.system.send_input_event(InputEvent::GamePadButtonDown {
							gamepad: id.into(),
							button,
						});
					}
				}
				gilrs::EventType::ButtonRepeated(_, _) => {}
				gilrs::EventType::ButtonReleased(button, _) => {
					if let Ok(button) = GamePadButton::try_from(button) {
						self.system
							.internal
							.gamepad_states
							.lock()
							.unwrap()
							.button_up(id.into(), button);
						self.system.send_input_event(InputEvent::GamePadButtonUp {
							gamepad: id.into(),
							button,
						});
					}
				}
				gilrs::EventType::ButtonChanged(_, _, _) => {}
				gilrs::EventType::AxisChanged(axis, value, _) => {
					if let Ok(axis) = GamePadAxis::try_from(axis) {
						self.system
							.internal
							.gamepad_states
							.lock()
							.unwrap()
							.set_axis_value(id.into(), axis, value);
						self.system
							.send_input_event(InputEvent::GamePadAxisChanged {
								gamepad: id.into(),
								axis,
								value,
							});
					}
				}
				gilrs::EventType::Connected => {
					self.system
						.send_input_event(InputEvent::GamePadConnected(id.into()));
				}
				gilrs::EventType::Disconnected => {
					self.system
						.send_input_event(InputEvent::GamePadDisconnected(id.into()));
				}
				_ => {}
			}
		}

		self.process_platform_events();
	}

	fn process_platform_events(&self) {
		for event in self.event_sub.collect() {
			match event {
				PlatformEvent::CursorMove { window, position } => {
					self.system.cursor_moved(window, position);
				}
				PlatformEvent::MouseButtonUp { window, button } => {
					self.system.mouse_up(window, button);
				}
				PlatformEvent::MouseButtonDown { window, button } => {
					self.system.mouse_down(window, button);
				}
				PlatformEvent::KeyUp {
					window,
					scancode,
					vkey,
					..
				} => {
					self.system.key_up(window, scancode, vkey);
				}
				PlatformEvent::KeyDown {
					window,
					scancode,
					vkey,
					..
				} => {
					self.system.key_down(window, scancode, vkey);
				}
				PlatformEvent::TextInput { window, text } => {
					self.system.text_input(window, text);
				}
				_ => (),
			}
		}
	}
}
