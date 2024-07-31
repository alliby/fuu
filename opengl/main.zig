const std = @import("std");
const gl = @import("gl");
const zstbi = @import("zstbi");
// const zm = @import("zmath");
const c = @cImport({
    @cDefine("GLFW_INCLUDE_NONE", "1");
    @cInclude("GLFW/glfw3.h");
});

const gl_log = std.log.scoped(.gl);
    
var procs: gl.ProcTable = undefined;

fn framebuffer_size_callback(window: ?*c.GLFWwindow, width: c_int, height: c_int) callconv(.C) void {
    _ = window;
    gl.Viewport(0, 0, width, height);
}

fn processInput(window: ?*c.GLFWwindow) void {
    if (c.glfwGetKey(window, c.GLFW_KEY_ESCAPE) == c.GLFW_PRESS) {
        c.glfwSetWindowShouldClose(window, @intFromBool(true));
    }
}

const vertices = [_]f32{
    // Position  // texture
    0.5,  0.5,   1.0, 1.0,
    0.5, -0.5,   1.0, 0.0,
   -0.5, -0.5,   0.0, 0.0,
   -0.5,  0.5,   0.0, 1.0
};

const indices = [_]i32 {
    0, 1, 3, // First Triangle
    1, 2, 3 // Second Triangle
};

const vertexShaderSource: [:0]const u8 =
    \\#version 330 core
    \\layout (location = 0) in vec2 aPos;
    \\layout (location = 1) in vec2 aTexCoord;
    \\out vec2 TextCoord;
    \\uniform mat4 transform;
    \\void main()
    \\{
    \\  gl_Position = transform * vec4(aPos, 0.0, 1.0);
    \\  TextCoord = aTexCoord;
    \\}
;

const fragmentShaderSource: [:0]const u8 =
    \\#version 330 core
    \\out vec4 FragColor;
    \\in vec2 TextCoord;
    \\uniform sampler2D Texture1;
    \\uniform sampler2D Texture2;
    \\void main()
    \\{
    \\  FragColor = mix(texture(Texture1, TextCoord), texture(Texture2, TextCoord), 0.2);
    \\}
;

pub fn main() !void {
    _ = c.glfwInit();
    c.glfwWindowHint(c.GLFW_CONTEXT_VERSION_MAJOR, gl.info.version_major);
    c.glfwWindowHint(c.GLFW_CONTEXT_VERSION_MINOR, gl.info.version_minor);
    c.glfwWindowHint(c.GLFW_OPENGL_PROFILE, c.GLFW_OPENGL_CORE_PROFILE);
    // c.glfwWindowHint(c.GLFW_SAMPLES, 4);
    // c.glfwWindowHint(c.GLFW_DECORATED, 0);
    // c.glfwWindowHint(c.GLFW_TRANSPARENT_FRAMEBUFFER, 1);

    const window = c.glfwCreateWindow(800, 600, "Learn OpenGl", null, null) orelse {
        std.debug.print("Failed to create GLFW window", .{});
        c.glfwTerminate();
        return error.CreateWindowFailed;
    };
    defer c.glfwTerminate();

    _ = c.glfwMakeContextCurrent(window);
    defer c.glfwMakeContextCurrent(null);
    _ = c.glfwSetFramebufferSizeCallback(window, framebuffer_size_callback);

    // Enable VSync to avoid drawing more often than necessary.
    c.glfwSwapInterval(1);

    if (!procs.init(c.glfwGetProcAddress)) {
        gl_log.err("failed to load OpenGL functions", .{});
        return error.InitFailed;
    }

    // Make the OpenGL procedure table current.
    gl.makeProcTableCurrent(&procs);
    defer gl.makeProcTableCurrent(null);
    
    // Compile our shader
    var success: c_int = undefined;
    var info_log_buf: [512:0]u8 = undefined;

    const vertexShader = gl.CreateShader(gl.VERTEX_SHADER);
    if (vertexShader == 0) return error.CreateVertexShaderFailed;
    defer gl.DeleteShader(vertexShader);

    gl.ShaderSource(vertexShader, 1, (&vertexShaderSource.ptr)[0..1], null);
    gl.CompileShader(vertexShader);
    gl.GetShaderiv(vertexShader, gl.COMPILE_STATUS, &success);
    if (success == gl.FALSE) {
        gl.GetShaderInfoLog(vertexShader, info_log_buf.len, null, &info_log_buf);
        gl_log.err("{s}", .{std.mem.sliceTo(&info_log_buf, 0)});
        return error.CompileVertexShaderFailed;
    }

    const fragmentShader = gl.CreateShader(gl.FRAGMENT_SHADER);
    if (fragmentShader == 0) return error.CreateFragmentShaderFailed;
    defer gl.DeleteShader(fragmentShader);

    gl.ShaderSource(fragmentShader, 1, (&fragmentShaderSource.ptr)[0..1], null);
    gl.CompileShader(fragmentShader);
    gl.GetShaderiv(fragmentShader, gl.COMPILE_STATUS, &success);
    if (success == gl.FALSE) {
        gl.GetShaderInfoLog(fragmentShader, info_log_buf.len, null, &info_log_buf);
        gl_log.err("{s}", .{std.mem.sliceTo(&info_log_buf, 0)});
        return error.CompileFragmentShaderFailed;
    }

    // link Shader
    const shaderProgram = gl.CreateProgram();
    gl.AttachShader(shaderProgram, vertexShader);
    gl.AttachShader(shaderProgram, fragmentShader);
    gl.LinkProgram(shaderProgram);
    defer gl.DeleteProgram(shaderProgram);

    // check for linking error
    gl.GetProgramiv(shaderProgram, gl.LINK_STATUS, &success);
    if (success == gl.FALSE) {
        gl.GetProgramInfoLog(shaderProgram, info_log_buf.len, null, &info_log_buf);
        gl_log.err("{s}", .{std.mem.sliceTo(&info_log_buf, 0)});
        return error.LinkShaderFailed;
    }

    // Bind the vertex array
    var vbo: c_uint = undefined;
    var vao: c_uint = undefined;
    var ebo: c_uint = undefined;
    gl.GenVertexArrays(1, (&vao)[0..1]);
    gl.GenBuffers(1, (&vbo)[0..1]);
    gl.GenBuffers(1, (&ebo)[0..1]);

    gl.BindVertexArray(vao);
    gl.BindBuffer(gl.ARRAY_BUFFER, vbo);
    gl.BufferData(gl.ARRAY_BUFFER, @sizeOf(@TypeOf(vertices)), &vertices, gl.STATIC_DRAW);
    gl.BindBuffer(gl.ELEMENT_ARRAY_BUFFER, ebo);
    gl.BufferData(gl.ELEMENT_ARRAY_BUFFER, @sizeOf(@TypeOf(indices)), &indices, gl.STATIC_DRAW);

    defer gl.DeleteVertexArrays(1, (&vao)[0..1]);
    defer gl.DeleteBuffers(1, (&vbo)[0..1]);
    defer gl.DeleteBuffers(1, (&ebo)[0..1]);

    gl.VertexAttribPointer(0, 2, gl.FLOAT, gl.FALSE, 4 * @sizeOf(f32), 0);
    gl.VertexAttribPointer(1, 2, gl.FLOAT, gl.FALSE, 4 * @sizeOf(f32), 2 * @sizeOf(f32));
    gl.EnableVertexAttribArray(0);
    gl.EnableVertexAttribArray(1);

    // load and create the textures 
    var textures : [2]c_uint = undefined;
    gl.GenTextures(2, &textures);
    defer gl.DeleteTextures(2, &textures);
    
    // Load the images from disk
    zstbi.init(std.heap.c_allocator);
    defer zstbi.deinit();
    zstbi.setFlipVerticallyOnLoad(true);
    var image_1 = try zstbi.Image.loadFromFile("src/container.jpg", 3);
    var image_2 = try zstbi.Image.loadFromFile("src/awesomeface.png", 4);
    const images = [2]*zstbi.Image { &image_1, &image_2 };
    defer inline for (&images) |image| { image.deinit(); };
    // Load the textures to opengl
    inline for (&images, textures, 0..) |image, texture, i| {
        const width: c_int = @intCast(image.width);
        const height: c_int = @intCast(image.height);
        const color = if (i==0) gl.RGB else gl.RGBA;
        gl.ActiveTexture(gl.TEXTURE0 + i);
        gl.BindTexture(gl.TEXTURE_2D, texture);
        gl.TexParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
        gl.TexParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.REPEAT);    
        gl.TexParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.TexParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.TexImage2D(gl.TEXTURE_2D, 0, color, width, height, 0, color, gl.UNSIGNED_BYTE, @ptrCast(image.data));
        gl.GenerateMipmap(gl.TEXTURE_2D);
    }

    gl.UseProgram(shaderProgram);
    gl.Uniform1i(gl.GetUniformLocation(shaderProgram, "Texture1"), 0);
    gl.Uniform1i(gl.GetUniformLocation(shaderProgram, "Texture2"), 1);

    const tranformLoc = gl.GetUniformLocation(shaderProgram, "transform");

    while (c.glfwWindowShouldClose(window) != 1) {
        // Handle input
        processInput(window);

        // render commands
        gl.ClearColor(0.2, 0.3, 0.3, 0.8);
        gl.Clear(gl.COLOR_BUFFER_BIT);

        // Apply transformations
        // const cos: f32 = @floatCast(@cos(c.glfwGetTime()));
        // const sin: f32 = @floatCast(@sin(c.glfwGetTime()));
        const transform = [_]f32 {
            0.5, 0.0, 0.0, 0.0,
            0.0, 0.5, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -0.6, 0.5, 0.0, 1.0,
        };
        gl.UniformMatrix4fv(tranformLoc, 1, gl.FALSE, &transform);
        
        // draw triangle
        gl.UseProgram(shaderProgram);
        gl.BindVertexArray(vao);
        gl.BindTexture(gl.TEXTURE_2D, textures[0]);
        gl.BindTexture(gl.TEXTURE_2D, textures[1]);        
        gl.DrawElements(gl.TRIANGLES, 6, gl.UNSIGNED_INT, 0);

        // poll events and swap the buffers
        c.glfwSwapBuffers(window);
        c.glfwPollEvents();
    }
}
