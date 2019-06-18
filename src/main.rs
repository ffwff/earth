extern crate kiss3d;
extern crate nalgebra as na;
extern crate ncollide3d as nc;

use kiss3d::resource::GLPrimitive;
use kiss3d::context::Context;
use kiss3d::camera::Camera;
use kiss3d::light::Light;
use kiss3d::resource::{Effect, Material, Mesh, ShaderAttribute, ShaderUniform, TextureManager, Texture};
use kiss3d::scene::ObjectData;
use kiss3d::window::Window;
use na::{Isometry3, Matrix3, Matrix4, Point2, Point3, Translation3, UnitQuaternion, Vector3};
use std::cell::RefCell;
use std::rc::Rc;
use std::path::Path;

fn main() {
    let mut window = Window::new("Kiss3d: cube");

    TextureManager::get_global_manager(|man| {
        man.add(&Path::new("./8081_earthmap4k.jpg"), "earth");
        man.add(&Path::new("./8081_earthbump4k.jpg"), "bump");
        man.add(&Path::new("./8081_earthspec4k.jpg"), "specular");
        man.add(&Path::new("./Clouds.jpg"), "cloud");
        man.add(&Path::new("./night_lights_modified.jpg"), "nights");
    });

    /*let atmo = window.add_sphere(1.1);
    atmo.set_material(Rc::new(RefCell::new(
        Box::new(AtmoMaterial::new()) as Box<Material + 'static>
    )));*/

    let mesh = nc::procedural::sphere(1.0, 128, 128, true);
    let mut c = window.add_trimesh(mesh, Vector3::new(1.0, 1.0, 1.0));
    c.set_material(Rc::new(RefCell::new(
        Box::new(NormalMaterial::new()) as Box<Material + 'static>
    )));

    window.set_light(Light::StickToCamera);

    while window.render() {
    }
}


// #region material
pub struct NormalMaterial {
    shader: Effect,
    position: ShaderAttribute<Point3<f32>>,
    normal: ShaderAttribute<Vector3<f32>>,
    tex_coord: ShaderAttribute<Point2<f32>>,
    view: ShaderUniform<Matrix4<f32>>,
    proj: ShaderUniform<Matrix4<f32>>,
    transform: ShaderUniform<Matrix4<f32>>,
    scale: ShaderUniform<Matrix3<f32>>,
    gl_texture_map: ShaderUniform<i32>,
    texture_map: Rc<Texture>,
    gl_height_map: ShaderUniform<i32>,
    height_map: Rc<Texture>,
    gl_specular_map: ShaderUniform<i32>,
    specular_map: Rc<Texture>,
    gl_cloud_map: ShaderUniform<i32>,
    cloud_map: Rc<Texture>,
    gl_night_map: ShaderUniform<i32>,
    night_map: Rc<Texture>,
    gl_camera_position: ShaderUniform<Point3<f32>>,
    gl_lightdir: ShaderUniform<Vector3<f32>>,
    time: f32,
    gl_time: ShaderUniform<f32>,
}

impl NormalMaterial {
    pub fn new() -> NormalMaterial {
        let mut shader = Effect::new_from_str(NORMAL_VERTEX_SRC, NORMAL_FRAGMENT_SRC);

        shader.use_program();

        fn get_map(s: &str) -> Rc<Texture> {
            let mut map: Option<Rc<Texture>> = None;
            TextureManager::get_global_manager(|man| {
                map = man.get(s);
            });
            map.unwrap()
        }

        NormalMaterial {
            position: shader.get_attrib("position").unwrap(),
            normal: shader.get_attrib("normal").unwrap(),
            tex_coord: shader.get_attrib("tex_coord").unwrap(),
            transform: shader.get_uniform("transform").unwrap(),
            scale: shader.get_uniform("scale").unwrap(),
            view: shader.get_uniform("view").unwrap(),
            proj: shader.get_uniform("proj").unwrap(),
            gl_camera_position: shader.get_uniform("camera_position").unwrap(),
            texture_map: get_map("earth"),
            gl_texture_map: shader.get_uniform("texture_map").unwrap(),
            height_map: get_map("bump"),
            gl_height_map: shader.get_uniform("height_map").unwrap(),
            specular_map: get_map("specular"),
            gl_specular_map: shader.get_uniform("specular_map").unwrap(),
            cloud_map: get_map("cloud"),
            gl_cloud_map: shader.get_uniform("cloud_map").unwrap(),
            night_map: get_map("nights"),
            gl_night_map: shader.get_uniform("night_map").unwrap(),
            gl_time: shader.get_uniform("time").unwrap(),
            gl_lightdir: shader.get_uniform("lightdir").unwrap(),
            time: 0.0,
            shader: shader,
        }
    }
}

impl Material for NormalMaterial {
    fn render(
        &mut self,
        pass: usize,
        transform: &Isometry3<f32>,
        scale: &Vector3<f32>,
        camera: &mut Camera,
        _: &Light,
        _: &ObjectData,
        mesh: &mut Mesh,
    ) {
        let ctx = Context::get();

        self.shader.use_program();
        self.position.enable();
        self.normal.enable();
        self.tex_coord.enable();

        camera.upload(pass, &mut self.proj, &mut self.view);
        self.gl_camera_position.upload(&camera.eye());

        let formated_transform = transform.to_homogeneous();
        let formated_scale = Matrix3::from_diagonal(&Vector3::new(scale.x, scale.y, scale.z));

        self.transform.upload(&formated_transform);
        self.scale.upload(&formated_scale);
        mesh.bind(&mut self.position, &mut self.normal, &mut self.tex_coord);
        mesh.bind_faces();

        ctx.active_texture(Context::TEXTURE0);
        ctx.bind_texture(Context::TEXTURE_2D, Some(&self.texture_map));
        self.gl_texture_map.upload(&0);

        ctx.active_texture(Context::TEXTURE1);
        ctx.bind_texture(Context::TEXTURE_2D, Some(&self.height_map));
        self.gl_height_map.upload(&1);

        ctx.active_texture(Context::TEXTURE0 + 2);
        ctx.bind_texture(Context::TEXTURE_2D, Some(&self.specular_map));
        self.gl_specular_map.upload(&2);

        ctx.active_texture(Context::TEXTURE0 + 3);
        ctx.bind_texture(Context::TEXTURE_2D, Some(&self.cloud_map));
        ctx.tex_parameteri(Context::TEXTURE_2D, Context::TEXTURE_WRAP_S, Context::REPEAT as i32);
        self.gl_cloud_map.upload(&3);

        ctx.active_texture(Context::TEXTURE0 + 4);
        ctx.bind_texture(Context::TEXTURE_2D, Some(&self.night_map));
        self.gl_night_map.upload(&4);

        self.time += 1.0;
        self.gl_time.upload(&self.time);

        let rot_axial = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), -23.4 / 360.0);
        let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), -self.time / 360.0);
        let pt = rot_axial.transform_point(&rot.transform_point(&Point3::new(0.0, 0.0, -1.0)));
        self.gl_lightdir.upload(&pt.coords);

        // post processing
        Context::get().draw_elements(
            Context::TRIANGLES,
            mesh.num_pts() as i32,
            Context::UNSIGNED_SHORT,
            0,
        );

        mesh.unbind();

        self.position.disable();
        self.normal.disable();
        self.tex_coord.disable();
    }
}

static NORMAL_VERTEX_SRC: &'static str = "#version 100
attribute vec3 position;
uniform mat4 view;
uniform mat4 proj;
uniform mat4 transform;
uniform mat3 scale;
uniform sampler2D height_map;

attribute vec3 normal;
varying vec3 ls_normal;
varying vec3 ls_position;

attribute vec2 tex_coord;
varying vec2 tex_coord_v;


void main() {
    ls_position = position;
    ls_normal   = normal;
    tex_coord_v = tex_coord;
    gl_Position = proj * view * transform * mat4(scale) * vec4(position, 1.0) +
        vec4(normal * texture2D(height_map, tex_coord_v.xy).x * 0.005, 1.0);
}
";

static NORMAL_FRAGMENT_SRC: &'static str = "#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif
varying vec3 ls_normal;
varying vec3 ls_position;
varying vec2 tex_coord_v;
uniform vec3 camera_position;
uniform sampler2D texture_map;
uniform sampler2D specular_map;
uniform sampler2D cloud_map;
uniform sampler2D night_map;
uniform vec3 lightdir;
uniform float time;
void main() {
    vec3 normal = normalize(ls_normal);
    float D = max(4.0*pow(dot(lightdir, normal), 40.0), 0.0);
    float R_out = 1.0 - dot(normalize(camera_position), normal); // CHANGEME
    float DN = dot(lightdir, normal);
    DN = clamp(DN * 2.0, 0.0, 1.0);

    vec4 land = texture2D(texture_map, tex_coord_v.xy);

    vec4 night_side = mix(
        texture2D(night_map, tex_coord_v.xy)*vec4(1.0,0.984,0.78,1.0),
        land,
        0.1
    );

    vec4 day_side = mix(
        mix(
            mix(
                land,
                vec4(0.93,0.92,0.90,1.0),
                texture2D(specular_map, tex_coord_v.xy) * D
            ),
            texture2D(cloud_map, vec2(tex_coord_v.x - (time / 5120.0), tex_coord_v.y) ),
            0.1
        ),
        vec4(0.471, 0.612, 0.831, 1.0),
        R_out
    );

    if (DN < 1.0 && DN > 0.0) {
        gl_FragColor = mix(
            mix(
                night_side,
                vec4(0.5, 0.0, 0.0, 1.0),
                DN * 0.1
            ),
            day_side,
            DN
        );
    } else {
        gl_FragColor = mix(
            night_side,
            day_side,
            DN
        );
    }
}
";

//#endregion

// #region atmo

//#endregion