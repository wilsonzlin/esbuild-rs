const fs = require('fs');
const path = require('path');

const esbuild = require('esbuild');
const esbuildNative = require('esbuild-native');

const TESTS_FILTER = new Set([
  'react.development',
  'c3',
  'three',
  'aws-sdk',
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

const textTestsMultiplied = tests.flatMap(({sourceText}) =>
  Array.from({length: ITERATIONS_PER_TEST}, () => sourceText)
);

const bufferTestsMultiplied = tests.flatMap(({sourceBuffer}) =>
  Array.from({length: ITERATIONS_PER_TEST}, () => sourceBuffer)
);

const testMinifier = async (minifierName, minifier) => {
  const start = Date.now();
  await minifier();
  const time = Date.now() - start;
  console.log(`${minifierName} took ${time} ms`);
};

(async () => {
  esbuildNative.startService();

  for (const {name, sourceBuffer, sourceText} of tests) {
    // First, ensure they produce identical output.
    const expected = (await esbuild.transform(sourceText, transformOptions)).js;
    const got = (await esbuildNative.minify([sourceBuffer]))[0].toString();
    if (expected !== got) {
      fs.writeFileSync('expected.js', expected);
      fs.writeFileSync('got.js', got);
      throw new Error(`esbuild-native output does not match esbuild for test ${name} (written to {expected,got}.js)`);
    }
  }

  const svc = await esbuild.startService();
  await testMinifier('esbuild', () => Promise.all(textTestsMultiplied.map(text => svc.transform(text, transformOptions))));
  svc.stop();

  await testMinifier('esbuild-native', () => esbuildNative.minify(bufferTestsMultiplied));
  esbuildNative.stopService();
})().catch(err => {
  console.error(err);
  process.exit(1);
});
