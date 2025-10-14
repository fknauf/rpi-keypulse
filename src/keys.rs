use evdev::KeyCode;
use std::collections::HashSet;

pub struct KeyClasses {
    dead_codes: HashSet<KeyCode>,
}

static ACTIVATION_KEY_COMBO: [KeyCode; 4] = [
    KeyCode::KEY_LEFTCTRL,
    KeyCode::KEY_LEFTALT,
    KeyCode::KEY_LEFTMETA,
    KeyCode::KEY_S,
];

static DEAD_KEYS: [KeyCode; 6] = [
    KeyCode::KEY_ESC,
    KeyCode::KEY_LEFTCTRL,
    KeyCode::KEY_LEFTALT,
    KeyCode::KEY_LEFTMETA,
    KeyCode::KEY_LEFTSHIFT,
    KeyCode::KEY_CAPSLOCK,
];

static NON_KBD_KEYS: [KeyCode; 107] = [
    KeyCode::BTN_0,
    KeyCode::BTN_1,
    KeyCode::BTN_2,
    KeyCode::BTN_3,
    KeyCode::BTN_4,
    KeyCode::BTN_5,
    KeyCode::BTN_6,
    KeyCode::BTN_7,
    KeyCode::BTN_8,
    KeyCode::BTN_9,
    KeyCode::BTN_LEFT,
    KeyCode::BTN_RIGHT,
    KeyCode::BTN_MIDDLE,
    KeyCode::BTN_SIDE,
    KeyCode::BTN_EXTRA,
    KeyCode::BTN_FORWARD,
    KeyCode::BTN_BACK,
    KeyCode::BTN_TASK,
    KeyCode::BTN_TRIGGER,
    KeyCode::BTN_THUMB,
    KeyCode::BTN_THUMB2,
    KeyCode::BTN_TOP,
    KeyCode::BTN_TOP2,
    KeyCode::BTN_PINKIE,
    KeyCode::BTN_BASE,
    KeyCode::BTN_BASE2,
    KeyCode::BTN_BASE3,
    KeyCode::BTN_BASE4,
    KeyCode::BTN_BASE5,
    KeyCode::BTN_BASE6,
    KeyCode::BTN_DEAD,
    KeyCode::BTN_SOUTH,
    KeyCode::BTN_EAST,
    KeyCode::BTN_C,
    KeyCode::BTN_NORTH,
    KeyCode::BTN_WEST,
    KeyCode::BTN_Z,
    KeyCode::BTN_TL,
    KeyCode::BTN_TR,
    KeyCode::BTN_TL2,
    KeyCode::BTN_TR2,
    KeyCode::BTN_SELECT,
    KeyCode::BTN_START,
    KeyCode::BTN_MODE,
    KeyCode::BTN_THUMBL,
    KeyCode::BTN_THUMBR,
    KeyCode::BTN_TOOL_PEN,
    KeyCode::BTN_TOOL_RUBBER,
    KeyCode::BTN_TOOL_BRUSH,
    KeyCode::BTN_TOOL_PENCIL,
    KeyCode::BTN_TOOL_AIRBRUSH,
    KeyCode::BTN_TOOL_FINGER,
    KeyCode::BTN_TOOL_MOUSE,
    KeyCode::BTN_TOOL_LENS,
    KeyCode::BTN_TOOL_QUINTTAP,
    KeyCode::BTN_TOUCH,
    KeyCode::BTN_STYLUS,
    KeyCode::BTN_STYLUS2,
    KeyCode::BTN_TOOL_DOUBLETAP,
    KeyCode::BTN_TOOL_TRIPLETAP,
    KeyCode::BTN_TOOL_QUADTAP,
    KeyCode::BTN_GEAR_DOWN,
    KeyCode::BTN_GEAR_UP,
    KeyCode::BTN_DPAD_UP,
    KeyCode::BTN_DPAD_DOWN,
    KeyCode::BTN_DPAD_LEFT,
    KeyCode::BTN_DPAD_RIGHT,
    KeyCode::BTN_TRIGGER_HAPPY1,
    KeyCode::BTN_TRIGGER_HAPPY2,
    KeyCode::BTN_TRIGGER_HAPPY3,
    KeyCode::BTN_TRIGGER_HAPPY4,
    KeyCode::BTN_TRIGGER_HAPPY5,
    KeyCode::BTN_TRIGGER_HAPPY6,
    KeyCode::BTN_TRIGGER_HAPPY7,
    KeyCode::BTN_TRIGGER_HAPPY8,
    KeyCode::BTN_TRIGGER_HAPPY9,
    KeyCode::BTN_TRIGGER_HAPPY10,
    KeyCode::BTN_TRIGGER_HAPPY11,
    KeyCode::BTN_TRIGGER_HAPPY12,
    KeyCode::BTN_TRIGGER_HAPPY13,
    KeyCode::BTN_TRIGGER_HAPPY14,
    KeyCode::BTN_TRIGGER_HAPPY15,
    KeyCode::BTN_TRIGGER_HAPPY16,
    KeyCode::BTN_TRIGGER_HAPPY17,
    KeyCode::BTN_TRIGGER_HAPPY18,
    KeyCode::BTN_TRIGGER_HAPPY19,
    KeyCode::BTN_TRIGGER_HAPPY20,
    KeyCode::BTN_TRIGGER_HAPPY21,
    KeyCode::BTN_TRIGGER_HAPPY22,
    KeyCode::BTN_TRIGGER_HAPPY23,
    KeyCode::BTN_TRIGGER_HAPPY24,
    KeyCode::BTN_TRIGGER_HAPPY25,
    KeyCode::BTN_TRIGGER_HAPPY26,
    KeyCode::BTN_TRIGGER_HAPPY27,
    KeyCode::BTN_TRIGGER_HAPPY28,
    KeyCode::BTN_TRIGGER_HAPPY29,
    KeyCode::BTN_TRIGGER_HAPPY30,
    KeyCode::BTN_TRIGGER_HAPPY31,
    KeyCode::BTN_TRIGGER_HAPPY32,
    KeyCode::BTN_TRIGGER_HAPPY33,
    KeyCode::BTN_TRIGGER_HAPPY34,
    KeyCode::BTN_TRIGGER_HAPPY35,
    KeyCode::BTN_TRIGGER_HAPPY36,
    KeyCode::BTN_TRIGGER_HAPPY37,
    KeyCode::BTN_TRIGGER_HAPPY38,
    KeyCode::BTN_TRIGGER_HAPPY39,
    KeyCode::BTN_TRIGGER_HAPPY40,
];

fn normalize_modifier(code: KeyCode) -> KeyCode {
    match code {
        KeyCode::KEY_RIGHTCTRL => KeyCode::KEY_LEFTCTRL,
        KeyCode::KEY_RIGHTALT => KeyCode::KEY_LEFTALT,
        KeyCode::KEY_RIGHTMETA => KeyCode::KEY_LEFTMETA,
        KeyCode::KEY_RIGHTSHIFT => KeyCode::KEY_LEFTSHIFT,
        other => other,
    }
}

impl KeyClasses {
    pub fn new(no_dead_keys: bool) -> KeyClasses {
        let dead_keys = if no_dead_keys {
            HashSet::new()
        } else {
            HashSet::from(DEAD_KEYS)
        };

        KeyClasses {
            dead_codes: HashSet::from(NON_KBD_KEYS)
                .union(&dead_keys)
                .cloned()
                .collect(),
        }
    }

    pub fn is_plopping(self: &Self, code: KeyCode) -> bool {
        let normalized_code = normalize_modifier(code);
        !self.dead_codes.contains(&normalized_code)
    }
}

pub struct KeyState {
    state: HashSet<KeyCode>,
    activation_combo: HashSet<KeyCode>,
}

impl KeyState {
    pub fn new() -> KeyState {
        KeyState {
            state: HashSet::new(),
            activation_combo: HashSet::from(ACTIVATION_KEY_COMBO),
        }
    }

    pub fn insert(self: &mut Self, code: KeyCode) {
        self.state.insert(normalize_modifier(code));
    }

    pub fn remove(self: &mut Self, code: KeyCode) {
        let normalized_code = normalize_modifier(code);
        self.state.remove(&normalized_code);
    }

    pub fn is_activation_combo(self: &Self) -> bool {
        self.state == self.activation_combo
    }
}
