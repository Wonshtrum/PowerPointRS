function unimplemented(name) {
    return () => console.log(name)
}

function get_memory_u8(ptr, size) {
    return new Uint8Array(wasm.instance.exports.memory.buffer, ptr, size);
}
function get_memory_f32(ptr, size) {
    return new Float32Array(wasm.instance.exports.memory.buffer, ptr, size);
}
function get_memory_u32(ptr, size) {
    return new Uint32Array(wasm.instance.exports.memory.buffer, ptr, size);
}
function copy_memory(ptr, size) {
    return wasm.instance.exports.memory.buffer.slice(ptr, ptr + size);
}

let wasm;
let memory;
let vb_ptr_size_dyn;
let vb_data_dyn;
let vb_data_const;
let presentation;
let subdata_ptr;
let subdata_slice;
let instances;

const DYNAMIC = 0;
const CONSTANT = 1;
const FLOAT_SIZE = 4;

let WIDTH = 120;
let HEIGHT = 90;
const SCALE = 16;
let canvas = document.getElementById("canvas");
let gl = canvas.getContext("webgl2", { premultipliedAlpha: false });
let vbo_dyn;

async function init() {
    memory = new WebAssembly.Memory({
        initial: 0,
        maximum: 1000,
        shared: true,
    });
    let wasm_path = "../target/wasm32-unknown-unknown/release/pptwasm.wasm";
    wasm = await WebAssembly.instantiateStreaming(fetch(wasm_path), {
        "js": {
            "memory": memory,
        },
        "pptrs": {
            "log": (ptr, size) => {
                let msg = new TextDecoder().decode(get_memory_u8(ptr, size));
                console.log(msg);
            },
            "error": (ptr, size) => {
                let msg = new TextDecoder().decode(get_memory_u8(ptr, size));
                console.error(msg);
            },
        }
    });

    let start = Date.now();
    presentation = wasm.instance.exports.main();
    let end = Date.now();
    console.log(presentation, (end - start) / 1000);

    vb_data_dyn = get_vbo(DYNAMIC);
    vb_data_const = get_vbo(CONSTANT);

    subdata_ptr = wasm.instance.exports.get_subdata_slice(presentation);
    subdata_slice = get_memory_u32(subdata_ptr, 3);
    instances = subdata_slice[2];

    WIDTH = wasm.instance.exports.get_width(presentation);
    HEIGHT = wasm.instance.exports.get_height(presentation);
    // wasm.instance.exports.display(presentation);
    init_gl();
}

function get_vbo(index) {
    let ptr = wasm.instance.exports.get_vbo_ptr(presentation, index);
    let size = wasm.instance.exports.get_vbo_size(presentation, index);
    console.log(ptr, size);
    let vbo;
    if (index == DYNAMIC) {
        vb_ptr_size_dyn = [ptr, size * 5];
        vbo = get_memory_f32(ptr, size * 5);
    } else if (index == CONSTANT) {
        // vbo = get_memory_f32(ptr, size * 1);
        vbo = Float32Array.from(get_memory_u8(ptr, size * 3 * FLOAT_SIZE), v => v / 255);
    }
    console.log(vbo);
    return vbo;
}

function click(x = 1, y = 1, n = 1) {
    console.log(x, y, n);
    let start = Date.now();
    wasm.instance.exports.click(presentation, x, y, n);
    console.log("click_time", Date.now() - start);
    // wasm.instance.exports.display(presentation);
    // console.log(vb_data_dyn);
    // console.log(vb_data_const);
}

function reset_backing_arraybuffer() {
    let [ptr, size] = vb_ptr_size_dyn;
    vb_data_dyn = get_memory_f32(ptr, size);
    subdata_slice = get_memory_u32(subdata_ptr, 3);
}

init();

const vertex_source = `\
#version 300 es
precision highp float;

layout (location=0) in vec4 a_position;
layout (location=1) in float a_visible;
layout (location=2) in vec3 a_color;

out vec4 v_color;

uniform vec2 u_viewport;

const vec2 vertices[6] = vec2[6](
    vec2(0, 0),
    vec2(1, 0),
    vec2(0, 1),

    vec2(0, 1),
    vec2(1, 0),
    vec2(1, 1)
);

void main() {
    vec2 vertex = vertices[gl_VertexID]*a_position.pq+a_position.xy;
    if (a_visible == 0.) {
        v_color = vec4(a_color, 0.0);
    } else {
        v_color = vec4(a_color, 1);
    }
    gl_Position = vec4(vec2(2, -2)*(vertex/u_viewport-0.5), 0, 1);
}
`;

const fragment_source = `\
#version 300 es
precision highp float;

in vec4 v_color;
out vec4 fragColor;

void main() {
    fragColor = v_color;
}
`;

function init_gl() {
    canvas.width = WIDTH * SCALE;
    canvas.height = HEIGHT * SCALE;

    if (!gl) {
        console.error("WebGL 2 not available");
    }

    gl.clearColor(1, 1, 1, 1);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.viewport(0, 0, canvas.width, canvas.height);

    let vertex_shader = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vertex_shader, vertex_source);
    gl.compileShader(vertex_shader);

    if (!gl.getShaderParameter(vertex_shader, gl.COMPILE_STATUS)) {
        console.error(gl.getShaderInfoLog(vertex_shader));
    }

    let fragment_shader = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fragment_shader, fragment_source);
    gl.compileShader(fragment_shader);

    if (!gl.getShaderParameter(fragment_shader, gl.COMPILE_STATUS)) {
        console.error(gl.getShaderInfoLog(fragment_shader));
    }

    let program = gl.createProgram();
    gl.attachShader(program, vertex_shader);
    gl.attachShader(program, fragment_shader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        console.error(gl.getProgramInfoLog(program));
    }

    gl.useProgram(program);
    let viewport_location = gl.getUniformLocation(program, "u_viewport");
    gl.uniform2f(viewport_location, WIDTH, HEIGHT);

    let vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    vbo_dyn = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo_dyn);
    gl.bufferData(gl.ARRAY_BUFFER, vb_data_dyn, gl.DYNAMIC_DRAW);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 5 * FLOAT_SIZE, 0);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribDivisor(0, 1);
    gl.vertexAttribPointer(1, 1, gl.UNSIGNED_BYTE, false, 5 * FLOAT_SIZE, 4 * FLOAT_SIZE);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribDivisor(1, 1);

    let vbo_const = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo_const);
    gl.bufferData(gl.ARRAY_BUFFER, vb_data_const, gl.STATIC_DRAW);
    gl.vertexAttribPointer(2, 3, gl.FLOAT, false, 12 * FLOAT_SIZE, 0);
    gl.enableVertexAttribArray(2);
    gl.vertexAttribDivisor(2, 1);

    render();
}

function update_vbo() {
    let update = subdata_slice[0];
    let start = subdata_slice[1] * 5;
    let end = subdata_slice[2] * 5;
    console.log(update, start, end);
    if (update && start <= end) {
        subdata_slice[0] = 1;
        gl.bindBuffer(gl.ARRAY_BUFFER, vbo_dyn);
        gl.bufferSubData(gl.ARRAY_BUFFER, start * 4, vb_data_dyn, start, 5 + end - start);
    }
}

function render() {
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, instances);
}

canvas.addEventListener("click", e => {
    let start = Date.now();
    let x = e.x - canvas.offsetLeft;
    let y = e.y - canvas.offsetTop;
    click(x * WIDTH / canvas.offsetWidth, y * HEIGHT / canvas.offsetHeight);
    reset_backing_arraybuffer();
    update_vbo();
    render();
    console.log("time", Date.now() - start);
});

function update() {
    reset_backing_arraybuffer();
    update_vbo();
    render();
}

function display() {
    wasm.instance.exports.display(presentation);
}