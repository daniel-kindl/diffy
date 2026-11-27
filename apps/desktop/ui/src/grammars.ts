// Syntax highlighting (C-07). Each import registers a language with a lazy
// loader: the grammar is a separate chunk that the page loads only when a file
// of that language is shown. The editor core does not grow by the grammars.
//
// This set covers the languages of languageOf (language.ts) that Monaco has a
// basic grammar for. JSON, TypeScript and JavaScript get a basic tokenizer
// here, not the language services of Monaco (those are not bundled).
import "monaco-editor/languages/definitions/rust/register";
import "monaco-editor/languages/definitions/typescript/register";
import "monaco-editor/languages/definitions/javascript/register";
import "monaco-editor/languages/definitions/markdown/register";
import "monaco-editor/languages/definitions/html/register";
import "monaco-editor/languages/definitions/css/register";
import "monaco-editor/languages/definitions/python/register";
import "monaco-editor/languages/definitions/go/register";
import "monaco-editor/languages/definitions/java/register";
import "monaco-editor/languages/definitions/cpp/register";
import "monaco-editor/languages/definitions/csharp/register";
import "monaco-editor/languages/definitions/shell/register";
import "monaco-editor/languages/definitions/yaml/register";
import "monaco-editor/languages/definitions/ini/register";
import "monaco-editor/languages/definitions/sql/register";
import "monaco-editor/languages/definitions/xml/register";
import "monaco-editor/languages/definitions/dockerfile/register";
