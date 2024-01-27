// import * as jsrpc from "json-rpc-2.0";
import { MonacoToProtocolConverter, ProtocolToMonacoConverter } from "monaco-languageclient";
import * as monaco from "monaco-editor-core";
import * as proto from "vscode-languageserver-protocol";
import contributes from "./contributes.json";
import config from "./language_configuration/solidity.configuration.json";

import Client from "./client";

export const monacoToProtocol = new MonacoToProtocolConverter(monaco);
export const protocolToMonaco = new ProtocolToMonacoConverter(monaco);

let language: null | Language;

export default class Language implements monaco.languages.ILanguageExtensionPoint {
  readonly id: string;
  readonly aliases: string[];
  readonly extensions: string[];
  readonly mimetypes: string[];

  private constructor(client: Client) {
    const { id, aliases, extensions, mimetypes } = Language.extensionPoint();
    this.id = id;
    this.aliases = aliases;
    this.extensions = extensions;
    this.mimetypes = mimetypes;
    this.registerLanguage(client);
  }

  static extensionPoint(): monaco.languages.ILanguageExtensionPoint & {
    aliases: string[];
    extensions: string[];
    mimetypes: string[];
  } {
    const id = contributes.contributes.languages[0].id;
    const aliases = contributes.contributes.languages[0].aliases;
    const extensions = contributes.contributes.languages[0].extensions;
    const mimetypes = ["text/x-solidity"]; // This is a common MIME type for Solidity, but you may need to adjust it

    return { id, extensions, aliases, mimetypes };

  }

  private registerLanguage(client: Client): void {
    void client;
    monaco.languages.register(Language.extensionPoint());
    monaco.languages.registerDocumentSymbolProvider(this.id, {
      // eslint-disable-next-line
      async provideDocumentSymbols(model, token): Promise<monaco.languages.DocumentSymbol[]> {
        void token;
        const response = await (client.request(proto.DocumentSymbolRequest.type.method, {
          textDocument: monacoToProtocol.asTextDocumentIdentifier(model),
        } as proto.DocumentSymbolParams) as Promise<proto.SymbolInformation[]>);

        const uri = model.uri.toString();

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.DocumentSymbol[] = protocolToMonaco.asSymbolInformations(response, uri);

        return result;
      },
    });


    monaco.languages.registerHoverProvider(this.id, {
      // eslint-disable-next-line
      async provideHover(model, position, token): Promise<monaco.languages.Hover> {
        void token;
        const response = await (client.request(proto.HoverRequest.type.method, {
          textDocument: monacoToProtocol.asTextDocumentIdentifier(model),
          position: monacoToProtocol.asPosition(position.column, position.lineNumber),
        } as proto.HoverParams) as Promise<proto.Hover>);

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.Hover = protocolToMonaco.asHover(response);





        return result;
      },
    });

    let mock_diagnostic: monaco.editor.IMarkerData = {
      severity: monaco.MarkerSeverity.Error,
      message: "This is a mock error",
      startLineNumber: 1,
      startColumn: 1,
      endLineNumber: 5,
      endColumn: 5,
    }

    //let diagnostic = monacoToProtocol.asDiagnostic(mock_diagnostic);


    monaco.editor.onDidCreateModel((model) => {

      //let diagnostic = client.diagnostic;

      //let markers = protocolToMonaco.asDiagnostics(diagnostic.diagnostics);

      //monaco.editor.setModelMarkers(model, "solidity", markers);

      model.onDidChangeContent((event) => {

        console.log("content changed", event);

        // get the diagnostics from the client and set them on the model
        let diagnostic = client.diagnostic;

        let markers = protocolToMonaco.asDiagnostics(diagnostic.diagnostics);

        monaco.editor.setModelMarkers(model, "solidity", markers);

      });



      //monaco.editor.setModelMarkers(model, "solidity", [mock_diagnostic]);
    });

    //monaco.editor.onDidChangeModelContent(async (event) => {

    //monaco.editor.setModelMarkers(model, "solidity", [mock_diagnostic]);






    /*monaco.languages.text(this.id, {
      // eslint-disable-next-line
      async provideCodeActions(model, range, context, token): Promise<monaco.languages.CodeActionList> {
        void token;
        const response = await (client.request(proto.DocumentDiagnosticRequest.type.method, {
          textDocument: monacoToProtocol.asTextDocumentIdentifier(model),
          range: monacoToProtocol.asRange(range),
          context: {
            diagnostics: [diagnostic],
          },
        } as proto.CodeActionParams) as Promise<proto.CodeAction>);
 
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const result: monaco.languages.CodeActionList = protocolToMonaco.asDiagnostic(response);
 
        console.log("Code action result", result);
 
        return result;
      },
    });*/



    monaco.languages.setMonarchTokensProvider('solidity', {
      // Tokenizer
      tokenizer: {
        root: [
          // natspec comments
          [/@\*\*[\s\S]*?\*\//, 'comment.doc'],

          // comments
          [/\/\/.*/, 'comment'],

          // operators
          [/[\+\-\*\%\=\/\!\&\|\^\<\>\~]/, 'operator'],

          // control keywords
          [/\b(if|else|for|while|do|break|continue|return|throw|try|catch|finally|switch|case|default)\b/, 'keyword.control'],

          // constants
          [/\b(true|false|null|undefined)\b/, 'constant.language'],

          // numbers
          [/\b\d+\b/, 'number'],

          // strings
          [/".*?"/, 'string'],

          // types
          [/\b(bool|byte|address|int|uint|string|mapping|array)\b/, 'keyword.type'],
        ],
      },
    });



  }

  static initialize(client: Client): Language {
    if (null == language) {
      language = new Language(client);
    } else {
      console.warn("Language already initialized; ignoring");
    }
    return language;
  }
}
