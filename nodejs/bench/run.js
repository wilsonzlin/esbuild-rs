const fs = require('fs');
const path = require('path');

const esbuild = require('esbuild');
const esbuildNative = require('esbuild-native');

const TESTS_FILTER = new Set([
  'react.development',
  'c3',
  'three',
  'plotly',
]);
const ITERATIONS_PER_TEST = 10;

const transformOptions = {
  minify: true,
  minifyIdentifiers: true,
  minifySyntax: true,
  minifyWhitespace: true,
};

const tests = [...TESTS_FILTER].map(f => {
  const buf = fs.readFileSync(path.join(__dirname, 'tests', f));
  return ({
    name: f,
    sourceBuffer: buf,
    sourceText: buf.toString(),
  });
});

const testMinifier = async (minifierName, minifier) => {
  const promises = [];
  const start = Date.now();
  for (const {sourceBuffer, sourceText} of tests) {
    for (let i = 0; i < ITERATIONS_PER_TEST; i++) {
      promises.push(minifier(sourceBuffer, sourceText));
    }
  }
  await Promise.all(promises);
  const time = Date.now() - start;
  console.log(`${minifierName} took ${time} ms`);
};

(async () => {
  esbuildNative.startService();

  for (const {name, sourceBuffer, sourceText} of tests) {
    // First, ensure they produce identical output.
    const expected = (await esbuild.transform(sourceText, transformOptions)).js;
    const got = (await esbuildNative.minify(sourceBuffer)).toString();
    if (expected !== got) {
      fs.writeFileSync('expected.js', expected);
      fs.writeFileSync('got.js', got);
      throw new Error(`esbuild-native output does not match esbuild for test ${name} (written to {expected,got}.js)`);
    }
  }

  const svc = await esbuild.startService();
  await testMinifier('esbuild', (_, text) => svc.transform(text, transformOptions));
  svc.stop();

  await testMinifier('esbuild-native', (buf, _) => esbuildNative.minify(buf));
  esbuildNative.stopService();
})().catch(err => {
  console.error(err);
  process.exit(1);
});
