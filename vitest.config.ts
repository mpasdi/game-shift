import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  plugins: [vue()], // 让 Vitest 能解析和编译 .vue 文件
  test: {
    environment: 'happy-dom', // 提供测试中的window document 环境
    include: ['src/**/*.test.ts'],
    clearMocks: true // 每个测试结束后的， 清理mock 的调用记录， 避免测试相互影响
  }
})
