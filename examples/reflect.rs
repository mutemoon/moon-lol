use bevy::{prelude::*, reflect::ReflectRef};
use serde::{Deserialize, Serialize};

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .register_type::<Movement>()
        .add_systems(Startup, startup)
        .add_systems(Update, dynamic_conditional_animation_system);

    app.update();

    app.should_exit();
}

#[derive(Component)]
struct Mark;

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
struct Movement {
    speed: f32,
}

fn startup(mut commands: Commands) {
    commands.spawn((Transform::default(), Movement { speed: 1.0 }, Mark));
}

fn dynamic_conditional_animation_system(
    world: &World,
    query: Query<Entity, With<Mark>>,
    registry: Res<AppTypeRegistry>,
) {
    let entity = world.get_entity(query.single().unwrap()).unwrap();

    let registry = registry.read();

    let type_registration = registry.get_with_short_type_path("Movement").unwrap();

    let reflect_component = type_registration.data::<ReflectComponent>().unwrap();

    let component = reflect_component.reflect(entity).unwrap();

    if let ReflectRef::Struct(struct_ref) = component.reflect_ref() {
        if let Some(speed_value) = struct_ref.get_field::<f32>("speed") {
            println!("成功获取 'speed' 字段的值: {}", speed_value);
            assert_eq!(*speed_value, 1.0);
        } else {
            println!("未找到 'speed' 字段");
        }
    }

    println!("component: {:?}", component);
}
