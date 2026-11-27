// The Monaco entry points have no type files of their own. The editor
// API has the same types as the package root. The feature modules are imported
// for their side effect only.
declare module "monaco-editor/editor/editor.api.js" {
  export * from "monaco-editor";
}
declare module "monaco-editor/editor/contrib/*";
declare module "monaco-editor/editor/standalone/browser/quickAccess/*";
declare module "monaco-editor/languages/definitions/*";
