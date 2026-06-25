const commitTypes = [
  'feat',
  'fix',
  'wip',
  'format',
  'style',
  'optimize',
  'refactor',
  'perf',
  'test',
  'build',
  'ci',
  'revert',
  'release',
  'docs',
  'other'
]

export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'subject-empty': [2, 'never'],
    'type-empty': [2, 'never'],
    'type-enum': [2, 'always', commitTypes]
  }
}
