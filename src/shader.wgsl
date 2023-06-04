
@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> mandelbulb: MandelBulb;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) screen_position: vec2<f32>,
};

struct Camera{
    ray_dir : mat3x3<f32>,
    position: vec3<f32>,
    ratio : f32,
    depth : f32,
};

struct MandelBulb{
    iterations: i32,
    max_ray_march_iterations: i32,
    collision_distance: f32,

    power : f32,

    color_map_black: vec3<f32>,
    color_map_white: vec3<f32>,
};

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 5.;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 5.;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.screen_position = vec2<f32>(x, y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray = pixel_to_ray(camera, in.screen_position);
    let steps = ray_march_steps_mandelbulb(ray, mandelbulb.max_ray_march_iterations, mandelbulb.collision_distance);
    let color = steps*mandelbulb.color_map_white + (1.-steps)*mandelbulb.color_map_black;
    return vec4<f32>(color.rgb, 1.0);
}

fn ray_march_steps_mandelbulb(ray: Ray, steps: i32, collision_distance: f32) -> f32 {
    var point = ray.origin;
    for (var i: i32 = 0; i < steps; i++) {
        let distance = distance_to_mandelbulb(point);
        if distance < collision_distance {
            return f32(i) / f32(steps);
        }
        point = point + ray.direction * distance;
    }
    return 1.;
}

fn pixel_to_ray(camera: Camera, pixel: vec2<f32>) -> Ray {
    let x = pixel.x * camera.ratio;
    let y = pixel.y;
    let direction = normalize(camera.ray_dir * vec3<f32>(x, y, camera.depth));
    return Ray(camera.position, direction);
}

fn distance_to_mandelbulb(point: vec3<f32>) -> f32 {
	var z = point;
	var dr : f32 = 1.0;
	var r : f32 = 0.0;
	for (var i : i32 = 0; i < mandelbulb.iterations ; i++) {
		r = length(z);
		if (r > 2.) {
            break;
        }

		// convert to polar coordinates
		var theta : f32 = acos(z.z/r);
		var phi : f32 = atan2(z.y,z.x);
        let power_min_1 = mandelbulb.power - 1.0;
		dr = pow(r, power_min_1)*mandelbulb.power*dr + 1.0;
		
		// scale and rotate the point
		let zr = pow(r,mandelbulb.power);
		theta = theta*mandelbulb.power;
		phi = phi*mandelbulb.power;
		
		// convert back to cartesian coordinates
		z = zr*vec3<f32>(sin(theta)*cos(phi), sin(phi)*sin(theta), cos(theta));
		z += point;
	}
	return 0.5*log(r)*r/dr;
}