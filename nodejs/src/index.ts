const esbuild = require('./esbuild.node');

export const minify = (codeUtf8: Buffer): Buffer => {
  return esbuild.minify(codeUtf8);
}
