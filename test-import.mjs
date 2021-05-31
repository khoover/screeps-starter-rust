import * as recast from 'recast';
import * as fs from 'fs';
const parse = recast.parse;
const print = recast.print;
const n = recast.types.namedTypes;
const b = recast.types.builders;

let code = fs.readFileSync('./rust-out/screeps_starter_rust.js', 'utf8');
let lines = code.split('\n');
lines = lines.filter((s) => { return !s.includes("new URL"); });
code = lines.join('\n');
console.log(code);
const ast = recast.parse(code);

let insideInit = false;
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
        console.log(path.node.declarations[0]);

        }
        return false;
    }
});

ast.program.body.unshift(
    b.importDeclaration(
        [
            b.importSpecifier(b.identifier('TextEncoder')),
            b.importSpecifier(b.identifier('TextDecoder'))
        ],
        b.literal("fastestsmallesttextencoderdecoder")
    )
);
console.log(recast.print(ast).code);
fs.writeFileSync('./rust-out/screeps_starter_rust.js', recast.print(ast).code, 'utf8');
