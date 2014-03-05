use std::mem;

use cow::btree::BTreeMap;
use snowmew::core::{Database, object_key};

use vertex_buffer::VertexBuffer;
use shader::Shader;

use ovr;
use Config;

static VS_SRC: &'static str =
"#version 150
uniform mat4 mat_model;
uniform mat4 mat_proj_view;

in vec3 in_position;
in vec2 in_texture;
in vec3 in_normal;

out vec3 fs_position;
out vec2 fs_texture;
out vec3 fs_normal;

void main() {
    gl_Position = mat_proj_view * mat_model * vec4(in_position, 1.);
    vec4 pos = mat_model * vec4(in_position, 1.);
    fs_position = pos.xyz / pos.w;
    fs_texture = in_texture;
    fs_normal = in_normal;
}
";

static BINDLESS_VS_INSTANCED_SRC: &'static str =
"#version 430
layout(location = 0) uniform mat4 mat_proj_view;
layout(location = 1) uniform int instance[512];

layout(std430, binding = 3) buffer MyBuffer
{
    mat4 model_matrix[];
};

in vec3 in_position;
in vec2 in_texture;
in vec3 in_normal;

out vec3 fs_position;
out vec2 fs_texture;
out vec3 fs_normal;

void main() {
    int id = instance[gl_InstanceID];
    gl_Position = mat_proj_view * model_matrix[id] * vec4(in_position, 1.);
    fs_position = model_matrix[id] * vec4(in_position, 1.);
    fs_texture = in_texture;
    fs_normal = in_normal;
    fs_material_id = material_id;
    fs_object_id = object_id;
}
";

static VS_PASS_SRC: &'static str =
"#version 400
in vec3 pos;

out vec2 TexPos;

void main() {
    gl_Position = vec4(pos.x, pos.y, 0.5, 1.);
    TexPos = vec2((pos.x+1)/2, (pos.y+1)/2); 
}
";

static VR_FS_SRC: &'static str = ovr::SHADER_FRAG_CHROMAB;

static FS_FLAT_SRC: &'static str =
"#version 400

uniform uint object_id;
uniform uint material_id;

in vec3 fs_position;
in vec2 fs_texture;
in vec3 fs_normal;

out vec4 out_position;
out vec2 out_uv;
out vec3 out_normal;
out vec4 out_material;

void main() {
    uint mask = 0xFFFF;
    out_position = vec4(fs_position, gl_FragCoord.z);
    out_uv = fs_texture;
    out_normal = fs_normal;
    out_material = vec4(float(material_id) / 65535., float(object_id) / 65535., 1., 1.);
}
";

static FS_DEFERED_SRC: &'static str =
"#version 400

uniform vec3 mat_color[128];

uniform sampler2D position;
uniform sampler2D uv;
uniform sampler2D normal;
uniform sampler2D pixel_drawn_by;

in vec2 TexPos;
out vec4 color;

void main() {
    int material = int(texture(pixel_drawn_by, TexPos).x * 65536.);
    if (material == 0) {
        color = vec4(0., 0., 0., 0.);
    } else {
        color = vec4(mat_color[material-1], 1.);
    }
}
";

#[deriving(Clone)]
pub struct Graphics
{
    last: Database,
    current: Database,
    vertex: BTreeMap<object_key, VertexBuffer>,

    flat_shader: Option<Shader>,
    flat_instanced_shader: Option<Shader>,

    defered_shader: Option<Shader>,

    ovr_shader: Option<Shader>,
}

impl Graphics
{
    pub fn new(db: Database) -> Graphics
    {
        Graphics {
            current: db.clone(),
            last: db,
            vertex: BTreeMap::new(),
            flat_shader: None,
            flat_instanced_shader: None,
            defered_shader: None,
            ovr_shader: None,

        }
    }

    pub fn update(&mut self, db: Database) -> Database
    {
        let mut db = db;
        mem::swap(&mut self.last, &mut self.current);
        mem::swap(&mut self.current, &mut db);
        db

    }

    fn load_vertex(&mut self, _: &Config)
    {
        for (oid, vbo) in self.current.walk_vertex_buffers()
        {
            match self.vertex.find(oid) {
                Some(_) => (),
                None => {
                    let vb = VertexBuffer::new(&vbo.vertex, vbo.index.as_slice());
                    self.vertex.insert(*oid, vb);
                }
            }
        }        
    }

    fn load_shaders(&mut self, cfg: &Config)
    {
        if self.ovr_shader.is_none() {
            self.ovr_shader = Some(
                Shader::new(VS_PASS_SRC, VR_FS_SRC,
                    &[(0, "pos")],
                    &[(0, "color")]
            ));
        }
        if self.flat_shader.is_none() {
            self.flat_shader = Some(
                Shader::new(VS_SRC, FS_FLAT_SRC,
                    &[(0, "in_position"), (1, "in_texture"), (2, "in_normal")],
                    &[(0, "out_position"), (1, "out_uv"), (2, "out_normal"), (3, "out_material")]
            ));
        }
        if self.defered_shader.is_none() {
            self.defered_shader = Some(Shader::new(VS_PASS_SRC, FS_DEFERED_SRC, &[], &[(0, "color")]));
        }
        if cfg.use_bindless() {
            if self.flat_instanced_shader.is_none() {
                self.flat_instanced_shader = Some(
                    Shader::new(BINDLESS_VS_INSTANCED_SRC, FS_FLAT_SRC,
                        &[(0, "in_position"), (1, "in_texture"), (2, "in_normal")],
                        &[(0, "out_position"), (1, "out_uv"), (2, "out_normal"), (3, "out_material")]
                ));
            }
        }
    }

    pub fn load(&mut self, cfg: &Config)
    {
        self.load_vertex(cfg);
        self.load_shaders(cfg);
    }
}