use raylib::prelude::*;

struct Player {
    position: Vector2,
    speed: f32,
    can_jump: bool,
}

struct EnvItem {
    rect: Rectangle,
    item_type: i32,
    color: Color,
}

const G: f32 = 900.0;
const PLAYER_JUMP_SPD: f32 = 450.0;
const PLAYER_HOR_SPD: f32 = 430.0;

type CameraUpdater =
    fn(&RaylibDrawHandle, &mut Camera2D, &Player, &[EnvItem], usize, f32, f32, f32);

fn main() {
    let mut camera_option = 0;
    let camera_updaters: [CameraUpdater; 5] = [
        updateCameraCenter,
        updateCameraCenterInsideMap,
        updateCameraCenterSmoothFollow,
        updateCameraEvenOutOnLanding,
        updateCameraPlayerBoundsPush,
    ];

    let screen_height = 450;
    let screen_width = 800;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("Example 2D Movement")
        .resizable()
        .build();

    rl.set_target_fps(60);

    let mut player = Player {
        position: Vector2::new(400.0, 280.0),
        speed: 0.0,
        can_jump: false,
    };

    let mut camera = Camera2D {
        target: Vector2::new(player.position.x + 20.0, player.position.y + 20.0),
        offset: Vector2::new(player.position.x, player.position.y),
        rotation: 0.0,
        zoom: 1.0,
    };

    let env_items = [
        EnvItem {
            rect: Rectangle::new(-1000.0, -1000.0, 3680.0, 3040.0),
            item_type: 0,
            color: Color::LIGHTGRAY,
        },
        EnvItem {
            rect: Rectangle::new(0.0, 400.0, 1000.0, 200.0),
            item_type: 1,
            color: Color::GRAY,
        },
        EnvItem {
            rect: Rectangle::new(300.0, 200.0, 400.0, 10.0),
            item_type: 1,
            color: Color::GRAY,
        },
        EnvItem {
            rect: Rectangle::new(250.0, 300.0, 100.0, 10.0),
            item_type: 1,
            color: Color::GRAY,
        },
        EnvItem {
            rect: Rectangle::new(650.0, 300.0, 100.0, 10.0),
            item_type: 1,
            color: Color::GRAY,
        },
    ];

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        if d.is_key_pressed(KeyboardKey::KEY_R) || player.position.x < -500.0 || player.position.x > 1500.0 || player.position.y > 1800.0 {
            player.position = Vector2::new(400.0, 280.0);
            player.speed = 0.0;
            player.can_jump = false;
        }

        if d.is_key_pressed(KeyboardKey::KEY_C) {
            camera_option = (camera_option + 1) % camera_updaters.len();
        }

        let delta_time = d.get_frame_time();

        camera.zoom += d.get_mouse_wheel_move() * 0.05;

        if camera.zoom > 3.0 {
            camera.zoom = 3.0;
        } else if camera.zoom < 0.5 {
            camera.zoom = 0.5;
        }

        println!("Player position: {:#?}", player.position);

        camera_updaters[camera_option](
            &d,
            &mut camera,
            &player,
            &env_items,
            env_items.len(),
            delta_time,
            screen_width as f32,
            screen_height as f32,
        );

        let player_ref: &mut Player = &mut player;

        updatePlayer(&d, player_ref, &env_items, delta_time);

        d.clear_background(Color::WHITE);
        {
            // println!("Using camera: {}", camera_option);
            let mut d2 = d.begin_mode2D(camera);
            for item in &env_items {
                d2.draw_rectangle_rec(item.rect, item.color);
            }

            let player_rect = Rectangle::new(
                player.position.x - 20.0,
                player.position.y - 40.0,
                40.0,
                40.0,
            );
            d2.draw_rectangle_rec(player_rect, Color::RED);

            d2.draw_circle_v(player.position, 5.0, Color::GOLD);
        }
    }
}

fn updateCameraCenter(
    _d: &RaylibDrawHandle,
    camera: &mut Camera2D,
    player: &Player,
    _env_items: &[EnvItem],
    _env_length: usize,
    _delta: f32,
    width: f32,
    height: f32,
) {
    camera.offset = Vector2::new(width / 2.0, height / 2.0);
    camera.target = player.position;
}

fn updateCameraCenterInsideMap(
    d: &RaylibDrawHandle,
    camera: &mut Camera2D,
    player: &Player,
    env_items: &[EnvItem],
    env_length: usize,
    delta: f32,
    width: f32,
    height: f32,
) {
    camera.target = player.position;
    camera.offset = Vector2::new(width / 2.0, height / 2.0);
    let mut min_x: f32 = 1000.0;
    let mut min_y: f32 = 1000.0;
    let mut max_x: f32 = -1000.0;
    let mut max_y: f32 = -1000.0;

    for i in 0..env_length {
        let ei = &env_items[i];
        min_x = min_x.min(ei.rect.x);
        max_x = max_x.max(ei.rect.x + ei.rect.width);
        min_y = min_y.min(ei.rect.y);
        max_y = max_y.max(ei.rect.y + ei.rect.height);
    }

    let max = d.get_world_to_screen2D(Vector2::new(max_x, max_y), *camera);
    let min = d.get_world_to_screen2D(Vector2::new(min_x, min_y), *camera);

    if max.x < width {
        camera.offset.x = width - (max.x - width / 2.0);
    }
    if max.y < height {
        camera.offset.y = height - (max.y - height / 2.0);
    }
    if min.x > 0.0 {
        camera.offset.x = width / 2.0 - min.x;
    }
    if min.y > 0.0 {
        camera.offset.y = height / 2.0 - min.y;
    }
}

fn updateCameraCenterSmoothFollow(
    _d: &RaylibDrawHandle,
    camera: &mut Camera2D,
    player: &Player,
    _env_items: &[EnvItem],
    _env_length: usize,
    delta: f32,
    width: f32,
    height: f32,
) {
    let minSpeed: f32 = 30.0;
    let minEffectLength: f32 = 10.0;
    let fractionSpeed: f32 = 0.8;

    camera.offset = Vector2::new(width / 2.0, height / 2.0);
    let mut diff = player.position - camera.target;
    let length: f32 = diff.length();

    if length > minEffectLength {
        let speed: f32 = minSpeed.max(fractionSpeed * length);
        diff.scale(speed * delta / length);
        camera.target += diff;
    }
}

fn updateCameraEvenOutOnLanding(
    _d: &RaylibDrawHandle,
    camera: &mut Camera2D,
    player: &Player,
    _env_items: &[EnvItem],
    _env_length: usize,
    delta: f32,
    width: f32,
    height: f32,
) {
    let even_out_speed: f32 = 700.0;
    let mut eveningOut: bool = false;
    let mut evenOutTarget: f32 = 0.0;

    camera.offset = Vector2::new(width / 2.0, height / 2.0);
    camera.target.x = player.position.x;

    if eveningOut {
        if evenOutTarget > camera.target.y {
            camera.target.y += even_out_speed * delta;

            if camera.target.y > evenOutTarget {
                camera.target.y = evenOutTarget;
                eveningOut = false;
            }
        } else {
            camera.target.y -= even_out_speed * delta;

            if camera.target.y < evenOutTarget {
                camera.target.y = evenOutTarget;
                eveningOut = false;
            }
        }
    } else {
        if player.can_jump && (player.speed == 0.0) && (player.position.y != camera.target.y) {
            eveningOut = true;
            evenOutTarget = player.position.y;
        }
    }
}

fn updateCameraPlayerBoundsPush(
    d: &RaylibDrawHandle,
    camera: &mut Camera2D,
    player: &Player,
    env_items: &[EnvItem],
    env_length: usize,
    delta: f32,
    width: f32,
    height: f32,
) {
    camera.offset = Vector2::new(width / 2.0, height / 2.0);
    camera.target = player.position;

    let bbox = Vector2::new(0.2, 0.2);

    let bbox_world_min = d.get_world_to_screen2D(Vector2::new((1.0 - bbox.x)*0.5*width, (1.0 - bbox.y)*0.5*height), *camera);
    let bbox_world_max = d.get_world_to_screen2D(Vector2::new((1.0 + bbox.x)*0.5*width, (1.0 + bbox.y)*0.5*height), *camera);

    camera.offset = Vector2::new((1.0 - bbox.x)*0.5 * width, (1.0 - bbox.y)*0.5*height);

    if player.position.x < bbox_world_min.x {
        camera.target.x = player.position.x;
    }
    if player.position.y < bbox_world_min.y {
        camera.target.y = player.position.y;
    }
    if player.position.x > bbox_world_max.x {
        camera.target.x = bbox_world_min.x + (player.position.x - bbox_world_max.x);
    }
    if player.position.y > bbox_world_max.y {
        camera.target.y = bbox_world_min.y + (player.position.y - bbox_world_max.y);
    }
}

fn updatePlayer(d: &RaylibDrawHandle, player: &mut Player, env_items: &[EnvItem], delta: f32) {
    if d.is_key_down(KeyboardKey::KEY_LEFT) {
        player.position.x -= PLAYER_HOR_SPD * delta;
    }
    if d.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.position.x += PLAYER_HOR_SPD * delta;
    }
    if d.is_key_down(KeyboardKey::KEY_SPACE) && player.can_jump {
        player.speed = -PLAYER_JUMP_SPD;
        player.can_jump = false;
    }

    let mut hitObstacle = false;
    for item in env_items {
        let p = &mut player.position;
        if item.item_type == 1
            && item.rect.x <= p.x
            && item.rect.x + item.rect.width >= p.x
            && item.rect.y >= p.y
            && item.rect.y <= p.y + player.speed * delta
        {
            hitObstacle = true;
            player.speed = 0.0;
            p.y = item.rect.y;
            break;
        }
    }

    if !hitObstacle {
        player.position.y += player.speed * delta;
        player.speed += G * delta;
        player.can_jump = false;
    } else {
        player.can_jump = true;
    }
}
