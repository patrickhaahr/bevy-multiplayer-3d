use bevy::prelude::*;

/// Marker component for the crosshair UI element
#[derive(Component)]
pub struct Crosshair;

/// Sets up the crosshair UI at the center of the screen
pub fn setup_crosshair(mut commands: Commands) {
    // Create a container for the crosshair positioned at the center
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            Crosshair,
        ))
        .with_children(|parent| {
            // Vertical line of the crosshair
            parent.spawn((
                Node {
                    width: Val::Px(2.0),
                    height: Val::Px(20.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));

            // Horizontal line of the crosshair
            parent.spawn((
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(2.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));

            // Center dot (optional, makes it easier to aim precisely)
            parent.spawn((
                Node {
                    width: Val::Px(4.0),
                    height: Val::Px(4.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
            ));
        });
}
