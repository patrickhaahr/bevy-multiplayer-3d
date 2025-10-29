use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cursor>()
            .add_systems(Update, update_cursor_locking)
            .add_systems(Startup, init_cursor_properties);
    }
}

#[derive(Resource, Default)]
pub struct Cursor {
    locked: bool,
}

impl Cursor {
    pub fn invert_lock(&mut self, cursor_options: &mut CursorOptions) {
        self.locked = !self.locked;
        if self.locked {
            cursor_options.grab_mode = CursorGrabMode::Locked;
            cursor_options.visible = false;
        } else {
            cursor_options.grab_mode = CursorGrabMode::None;
            cursor_options.visible = true;
        }
    }
}

fn init_cursor_properties(
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut cursor: ResMut<Cursor>,
) {
    if let Ok(mut cursor_options) = cursor_query.single_mut() {
        cursor.invert_lock(&mut cursor_options);
    }
}

fn update_cursor_locking(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor: ResMut<Cursor>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if let Ok(mut cursor_options) = cursor_query.single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            cursor.invert_lock(&mut cursor_options);
        }
    }
}
