const fs = require('fs');
const path = require('path');
const vega = require('vega');
const vegaLite = require('vega-lite');

const esbuild = require('esbuild');
const esbuildNative = require('..');

const TESTS_FILTER = new Set([
  'react.development',
  'c3',
  'three',
  'aws-sdk',
  'plotly',
]);
const GOAL_BYTES = 500 * 1024 * 1024;

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
    size: buf.length,
    sourceBuffer: buf,
    sourceText: buf.toString(),
  });
});

(async () => {
  esbuildNative.startService();
  const esbuildService = await esbuild.startService();

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

  const minifiers = [
    ['esbuild', (_, text) => esbuildService.transform(text, transformOptions)],
    ['esbuild-native', (buf, _) => esbuildNative.minify(buf)],
  ];
  const results = [];

  for (const [minifierName, minifier] of minifiers) {
    for (const {name, size, sourceBuffer, sourceText} of tests) {
      const promises = [];
      for (let i = 0; i < Math.ceil(GOAL_BYTES / size); i++) {
        promises.push(minifier(sourceBuffer, sourceText));
      }
      const start = Date.now();
      await Promise.all(promises);
      results.push({
        test: name,
        minifier: minifierName,
        time: Date.now() - start,
      });
    }
  }

  esbuildService.stop();
  esbuildNative.stopService();

  const chartSpec = vegaLite.compile({
    $schema: 'https://vega.github.io/schema/vega-lite/v4.json',
    data: {
      values: results,
    },
    height: 400,
    width: 150,
    mark: 'bar',
    encoding: {
      column: {
        field: 'test',
        type: 'nominal',
        sort: tests.map(t => t.name),
        axis: {title: ''},
      },
      y: {
        field: 'time',
        type: 'quantitative',
      },
      x: {
        field: 'minifier',
        type: 'nominal',
        axis: {title: ''},
      },
      color: {
        field: 'minifier',
        type: 'nominal',
        scale: {range: ['#05E0E9', '#CFEED1']},
      },
    },
    config: {
      view: {stroke: 'transparent'},
      axis: {domainWidth: 1},
      legend: {
        disable: true,
      },
    },
  }).spec;
  const chartRt = vega.parse(chartSpec);
  const chartView = new vega.View(chartRt).renderer('svg');
  const svg = await chartView.toSVG();
  fs.writeFileSync(path.join(__dirname, 'results.svg'), svg);
})().catch(err => {
  console.error(err);
  process.exit(1);
});
