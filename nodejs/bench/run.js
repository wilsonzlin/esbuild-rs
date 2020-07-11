const benchmark = require('benchmark');
const fs = require('fs');
const path = require('path');

const esbuild = require('esbuild');
const esbuildNative = require('esbuild-native');

const tests = fs.readdirSync(path.join(__dirname, 'tests')).map(f => ({
  name: f,
  sourceBuffer: fs.readFileSync(path.join(__dirname, 'tests', f)),
  sourceText: fs.readFileSync(path.join(__dirname, 'tests', f), 'utf8'),
}));

const suite = new benchmark.Suite();
suite.add('esbuild', () => {
  tests.forEach(({name, sourceText}) => esbuild.transformSync(sourceText, {
    minify: true,
    minifyIdentifiers: true,
    minifySyntax: true,
    minifyWhitespace: true,
  }));
});
suite.add('esbuild-native', () => {
  tests.forEach(({name, sourceBuffer}) => esbuildNative.minify(sourceBuffer));
});
suite
  .on('complete', () => suite.forEach(({name, hz}) => console.log(name, hz)))
  .on('error', console.error)
  .run();
