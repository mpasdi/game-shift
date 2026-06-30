export default {
  useEmoji: true,
  emojiAlign: 'center',
  scopes: ['app', 'ui', 'games', 'scanner', 'launcher', 'db', 'docs', 'config'],
  types: [
    { value: 'feat', name: '特性:   🚀  新增功能', emoji: '🚀' },
    { value: 'fix', name: '修复:   🧩  修复缺陷', emoji: '🧩' },
    { value: 'wip', name: '待续:   🤣  功能开发中', emoji: '🤣' },
    {
      value: 'format',
      name: '格式:   🎨  代码格式（不影响功能，例如空格、分号等格式修正）',
      emoji: '🎨'
    },
    {
      value: 'style',
      name: '样式:   🧣  ui样式（不影响功能，针对ui的改造/css的变更）',
      emoji: '🧣'
    },
    {
      value: 'optimize',
      name: '优化:   ✨  功能优化',
      emoji: '✨'
    },
    {
      value: 'refactor',
      name: '重构:   ♻️  代码重构（不包括 bug 修复、功能新增）',
      emoji: '♻️'
    },
    { value: 'perf', name: '性能:   ⚡️  性能优化', emoji: '⚡️' },
    {
      value: 'test',
      name: '测试:   ✅  添加疏漏测试或已有测试改动',
      emoji: '✅'
    },
    {
      value: 'build',
      name: '构建:   📦  构建流程、外部依赖变更（如升级 npm 包、修改 webpack 配置等）',
      emoji: '📦️'
    },
    { value: 'ci', name: '集成:   🎡  修改 CI 配置、脚本', emoji: '🎡' },
    { value: 'revert', name: '回退:   ⏪️  回滚 commit', emoji: '⏪️' },
    { value: 'release', name: '发版:   🎉  加鸡腿', emoji: '🎉' },
    { value: 'docs', name: '文档:   📚  文档变更', emoji: '📚' },
    {
      value: 'other',
      name: '其他:   🔨  对构建过程或辅助工具和库的更改（不影响源文件、测试用例）',
      emoji: '🔨'
    }
  ]
}
