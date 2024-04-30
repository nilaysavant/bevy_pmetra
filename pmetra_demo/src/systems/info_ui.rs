use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_pmetra::{
    pmetra_plugins::components::camera::CadCamera,
    re_exports::bevy_mod_picking::picking_core::Pickable,
};
use itertools::Itertools;

#[derive(Debug, Clone, Reflect)]
pub enum KeyType {
    MouseKey(MouseButton),
    MouseScroll,
    KeyboardKey(KeyCode),
}

#[derive(Debug, Clone, Reflect, Component)]
pub struct ShortcutInfo {
    pub shortcut: Vec<KeyType>,
    pub description: String,
}

pub fn get_shortcuts_info() -> [ShortcutInfo; 6] {
    [
        ShortcutInfo {
            shortcut: vec![KeyType::MouseKey(MouseButton::Left)],
            description: "Select".to_string(),
        },
        ShortcutInfo {
            shortcut: vec![KeyType::MouseKey(MouseButton::Right)],
            description: "Orbit".to_string(),
        },
        ShortcutInfo {
            shortcut: vec![
                KeyType::KeyboardKey(KeyCode::ShiftLeft),
                KeyType::MouseKey(MouseButton::Right),
            ],
            description: "Pan".to_string(),
        },
        ShortcutInfo {
            shortcut: vec![KeyType::MouseScroll],
            description: "Zoom".to_string(),
        },
        ShortcutInfo {
            shortcut: vec![KeyType::KeyboardKey(KeyCode::Space)],
            description: "Fire".to_string(),
        },
        ShortcutInfo {
            shortcut: vec![KeyType::KeyboardKey(KeyCode::F2)],
            description: "Debug".to_string(),
        },
    ]
}

pub fn setup_info_ui(mut commands: Commands, cameras: Query<Entity, Added<CadCamera>>) {
    if cameras.is_empty() {
        // Wait for a camera to be added.
        return;
    }
    debug!("Spawning Info UI...");
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    // fill the entire window
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.), // place at the bottom of the window
                    padding: UiRect::all(Val::Px(5.)),
                    ..Default::default()
                },
                ..Default::default()
            },
            Pickable::IGNORE, // Ignore picking events on the UI.
        ))
        .id();

    let shortcuts_info = get_shortcuts_info();
    for shortcut_info in shortcuts_info {
        let ShortcutInfo {
            shortcut,
            description,
        } = &shortcut_info;
        let shortcut_label = shortcut
            .iter()
            .map(|key_type| match key_type {
                KeyType::MouseKey(mouse_button) => {
                    format!("Mouse{:?}", mouse_button)
                }
                KeyType::KeyboardKey(key_code) => {
                    format!("{:?}", key_code)
                }
                KeyType::MouseScroll => "Scroll".to_string(),
            })
            .join(" + ");
        let shortcut_info = commands
            .spawn((
                TextBundle {
                    text: Text::from_section(
                        format!("[{}: {}]", description, shortcut_label),
                        TextStyle {
                            font_size: 16.,
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                    // Set the alignment of the Text
                    .with_justify(JustifyText::Center)
                    .with_no_wrap(),
                    // Set the style of the TextBundle itself.
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::all(Val::Px(5.)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::BLACK.with_a(0.8)),
                    ..default()
                },
                shortcut_info,
                Pickable::IGNORE, // Ignore picking events on the UI.
            ))
            .id();
        commands.entity(root).add_child(shortcut_info);
    }
}

pub fn update_info_ui(
    mut shortcuts_info: Query<(Entity, &ShortcutInfo, &mut Text)>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mouse_wheel: EventReader<MouseWheel>,
) {
    let mut shortcuts_pressed_info = shortcuts_info
        .iter()
        .map(|(entity, info, _)| {
            let is_pressed = info.shortcut.iter().all(|s| match s {
                KeyType::MouseKey(mouse_button) => mouse_input.pressed(*mouse_button),
                KeyType::KeyboardKey(key_code) => key_input.pressed(*key_code),
                KeyType::MouseScroll => !mouse_wheel.is_empty(),
            });
            (entity, info.clone(), is_pressed)
        })
        .collect::<Vec<_>>();
    let contains_orbit = shortcuts_pressed_info
        .iter()
        .any(|(_, s, is_pressed)| *is_pressed && s.description == "Orbit");
    let contains_pan = shortcuts_pressed_info
        .iter()
        .any(|(_, s, is_pressed)| *is_pressed && s.description == "Pan");
    if contains_orbit && contains_pan {
        // False the orbit shortcut if pan is pressed.
        shortcuts_pressed_info
            .iter_mut()
            .for_each(|(_, s, is_pressed)| {
                if s.description == "Orbit" {
                    *is_pressed = false;
                }
            });
    }
    // Update the color of the shortcut text based on the pressed state...
    for (entity, _, is_pressed) in shortcuts_pressed_info.iter() {
        let Ok((_, _, mut text)) = shortcuts_info.get_mut(*entity) else {
            continue;
        };
        if *is_pressed {
            text.sections[0].style.color = Color::YELLOW;
        } else {
            text.sections[0].style.color = Color::WHITE;
        }
    }
}
