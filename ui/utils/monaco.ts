import { Monaco } from '@monaco-editor/react';

export const registerSkriptLanguage = (monaco: Monaco) => {
  // Register Skript language
  monaco.languages.register({ id: 'skript' });

  // Define tokens for Skript
  monaco.languages.setMonarchTokensProvider('skript', {
    defaultToken: '',
    tokenPostfix: '.sk',

    keywords: [
      'on', 'command', 'trigger', 'function', 'if', 'else', 'loop', 'every', 
      'while', 'stop', 'exit', 'return', 'cancel', 'event', 'message', 
      'broadcast', 'send', 'set', 'add', 'remove', 'give', 'teleport', 
      'kill', 'spawn', 'execute', 'wait', 'chance', 'random', 'player', 
      'victim', 'attacker', 'target', 'location', 'world', 'metadata', 
      'variable', 'list', 'index', 'is', 'isn\'t', 'not', 'and', 'or', 
      'has', 'contains', 'greater', 'less', 'than', 'equal', 'to'
    ],

    operators: [
      '=', '<', '>', '!', ':', '.'
    ],

    // Symbols
    symbols: /[=><!~?:&|+\-*\/\^%]+/,

    tokenizer: {
      root: [
        // Keywords and identifiers
        [/[a-zA-Z_]\w*/, {
          cases: {
            '@keywords': 'keyword',
            '@default': 'identifier'
          }
        }],

        // Comments
        [/#.*$/, 'comment'],

        // Strings
        [/"/, { token: 'string.quote', bracket: '@open', next: '@string' }],

        // Variables
        [/\{[^\s\}]+\}/, 'variable'],

        // Numbers
        [/\d+(\.\d+)?/, 'number'],

        // Whitespace
        { include: '@whitespace' },

        // Delimiters
        [/[{}()\[\]]/, '@brackets'],
        [/[<>](?!@symbols)/, '@brackets'],
        [/@symbols/, {
          cases: {
            '@operators': 'operator',
            '@default': ''
          }
        }],
      ],

      whitespace: [
        [/[ \t\r\n]+/, 'white'],
      ],

      string: [
        [/[^\\"]+/, 'string'],
        [/\\./, 'string.escape'],
        [/"/, { token: 'string.quote', bracket: '@close', next: '@pop' }],
      ],
    },
  });

  // Define configuration for Skript
  monaco.languages.setLanguageConfiguration('skript', {
    comments: {
      lineComment: '#',
    },
    brackets: [
      ['{', '}'],
      ['[', ']'],
      ['(', ')'],
    ],
    autoClosingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
    ],
    surroundingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
    ],
  });
};

export const getLanguageFromExtension = (filename: string | null): string => {
  if (!filename) return 'yaml';
  const ext = filename.split('.').pop()?.toLowerCase();
  switch (ext) {
    case 'json': return 'json';
    case 'yml':
    case 'yaml': return 'yaml';
    case 'sk': return 'skript';
    case 'toml': return 'toml';
    case 'lua': return 'lua';
    case 'md': return 'markdown';
    case 'properties': return 'ini'; // Monaco uses ini for properties usually
    case 'sh': return 'shell';
    case 'js': return 'javascript';
    case 'xml': return 'xml';
    case 'html': return 'html';
    case 'css': return 'css';
    case 'txt':
    case 'log': return 'plaintext';
    default: return 'yaml';
  }
};
