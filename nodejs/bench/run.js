const benchmark = require('benchmark');
const fs = require('fs');
const path = require('path');

const esbuild = require('esbuild');
const esbuildNative = require('esbuild-native');

const transformOptions = {
  minify: true,
  minifyIdentifiers: true,
  minifySyntax: true,
  minifyWhitespace: true,
};

const tests = fs.readdirSync(path.join(__dirname, 'tests')).map(f => ({
  name: f,
  sourceBuffer: fs.readFileSync(path.join(__dirname, 'tests', f)),
  sourceText: fs.readFileSync(path.join(__dirname, 'tests', f), 'utf8'),
}));

// First, ensure they produce identical output.
for (const {name, sourceBuffer, sourceText} of tests) {
  const expected = esbuild.transformSync(sourceText, transformOptions).js;
  const got = esbuildNative.minify(sourceBuffer).toString();
  if (expected !== got) {
    fs.writeFileSync('expected.js', expected);
    fs.writeFileSync('got.js', got);
    throw new Error(`esbuild-native output does not match esbuild for test ${name} (written to {expected,got}.js)`);
  }
}

const suite = new benchmark.Suite();
suite.add('esbuild', () => {
  tests.forEach(({name, sourceText}) => esbuild.transformSync(sourceText, transformOptions));
});
suite.add('esbuild-native', () => {
  tests.forEach(({name, sourceBuffer}) => esbuildNative.minify(sourceBuffer));
});
suite
  .on('complete', () => suite.forEach(({name, hz}) => console.log(name, hz)))
  .on('error', console.error)
  .run();
