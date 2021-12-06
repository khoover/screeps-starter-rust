const spawn = require('child_process').spawn;
const { src, dest, series, parallel } = require('gulp');
const through2 = require('through2');
const recast = require('recast');
const fs = require('fs');
const { Transform } = require('stream');
const rollup = require('rollup');
const node_resolve = require('@rollup/plugin-node-resolve');
const del = require('del');

const parse = recast.parse;
const print = recast.print;
const n = recast.types.namedTypes;
const b = recast.types.builders;

function wasm_pack() {
    return spawn('rustup', ['run', 'nightly', 'wasm-pack', 'build', '--target', 'web', '--release', '--out-dir', 'rust-out', '--no-typescript'], { shell: '/bin/bash', stdio: 'inherit' });
}

function parse_generated_js(code) {
    let lines = code.split('\n');
    lines = lines.filter((s) => { return !s.includes("new URL"); });
    code = lines.join('\n');
    const ast = recast.parse(code);

    let insideInit = false;
    let imported_polyfill = false;
    recast.visit(ast, {
        visitFunction(path) {
            const node = path.node;
            if (node.id && node.id.name === 'load') {
                path.prune();
            } else if (node.id && node.id.name === 'init') {
                path.get('async').replace(false);
                path.get('params', 0, 'name').replace('module');
                insideInit = true;
                this.traverse(path);
                insideInit = false;
                return;
            }
            return false;
        },
        visitIfStatement(path) {
            if (!insideInit) return false;
            path.prune();
            return false;
        },
        visitVariableDeclaration(path) {
            if (!insideInit) return false;
            const node = path.node;
            if (node.declarations.some((d) => {
                return d.type === "VariableDeclarator" &&
                    d.init.type === "AwaitExpression";
            })) {
                path.get('declarations').replace([
                    b.variableDeclarator(
                        b.identifier("instance"),
                        b.newExpression(
                            b.identifier('WebAssembly.Instance'),
                            [b.identifier('module'), b.identifier('imports')]
                        )
                    )
                ]);
            }
            return false;
        },
        visitImportDeclaration(path) {
            if (path.node.source.value === "../EncoderDecoderTogether.min.js") imported_polyfill = true;
            return false;
        }
    });

    if (!imported_polyfill) ast.program.body.unshift(
        b.importDeclaration(
            [
            /*    b.importSpecifier(b.identifier('TextEncoder')),
                b.importSpecifier(b.identifier('TextDecoder'))*/
            ],
            //b.literal('util')
            b.literal("../EncoderDecoderTogether.min.js")
        )
    );
    return recast.print(ast).code;
}

function fix_generated_code() {
    return src('rust-out/screeps_starter_rust.js')
        .pipe(through2({objectMode: true},
            (chunk, _, cb) => {
                let source = chunk.contents.toString('utf8');
                source = parse_generated_js(source);
                chunk.contents = Buffer.from(source, 'utf8');
                cb(null, chunk);
            }
        ))
        .pipe(dest('rust-out'));
}

async function run_rollup() {
    const bundle = await rollup.rollup({
        input: './javascript/main.js',
        plugins: [
            node_resolve.nodeResolve()
        ]
    });
    await bundle.write({
        format: 'cjs',
        dir: 'dist'
    });
}

exports.clean = () => {
    return del(['dist', 'rust-out']);
}

function move_wasm() {
    return src('rust-out/**/*.wasm')
        .pipe(dest('dist'));
}

exports.default = series(wasm_pack, parallel(series(fix_generated_code, run_rollup), move_wasm));
