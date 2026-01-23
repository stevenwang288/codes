/**
 * 语言包加载器
 */

import { I18nManager, TranslationMap } from './I18nManager.js';
import { readdir, readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

export interface LoaderOptions {
  baseDir?: string;
  autoLoad?: boolean;
}

export class I18nLoader {
  private baseDir: string;
  private manager: I18nManager;

  constructor(manager: I18nManager, options: LoaderOptions = {}) {
    this.manager = manager;
    this.baseDir = options.baseDir || I18nLoader.getDefaultLocalesDir();
  }

  private static getDefaultLocalesDir(): string {
    const thisFile = fileURLToPath(import.meta.url);
    const thisDir = path.dirname(thisFile);

    // repoRoot/i18n/core/Loader.ts -> repoRoot/locales
    return path.resolve(thisDir, '../../locales');
  }

  /**
   * 加载指定语言的所有语言包
   */
  async loadLocale(locale: string): Promise<void> {
    try {
      // 加载通用语言包
      const commonTranslations = await this.loadJsonFile(`${this.baseDir}/${locale}/common.json`);
      
      // 加载插件特定语言包
      const pluginTranslations = await this.loadPluginTranslations(locale);
      
      // 合并所有翻译
      const mergedTranslations: TranslationMap = {
        ...commonTranslations,
        ...pluginTranslations
      };
      
      await this.manager.loadLocale(locale, mergedTranslations);
    } catch (error) {
      console.error(`Failed to load locale "${locale}":`, error);
      throw error;
    }
  }

  /**
   * 加载所有可用语言
   */
  async loadAllLocales(): Promise<string[]> {
    const locales = await this.getAvailableLocales();
    
    for (const locale of locales) {
      await this.loadLocale(locale);
    }
    
    return locales;
  }

  /**
   * 自动加载所有语言包
   */
  async autoLoad(): Promise<void> {
    await this.loadAllLocales();
  }

  /**
   * 获取可用语言列表
   */
  private async getAvailableLocales(): Promise<string[]> {
    const entries = await readdir(this.baseDir, { withFileTypes: true });
    return entries
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name);
  }

  /**
   * 加载插件特定翻译
   */
  private async loadPluginTranslations(locale: string): Promise<TranslationMap> {
    const localeDir = path.join(this.baseDir, locale);
    const entries = await readdir(localeDir, { withFileTypes: true });
    const pluginTranslations: TranslationMap = {};

    for (const entry of entries) {
      if (!entry.isFile()) continue;
      if (!entry.name.endsWith('.json')) continue;
      if (entry.name === 'common.json') continue;

      const filePath = path.join(localeDir, entry.name);
      const translations = await this.loadJsonFile(filePath);

      // Plugin files are already namespaced by their top-level keys.
      // Example: { "pty": { ... } }, { "shellStrategy": { ... } }
      Object.assign(pluginTranslations, translations);
    }

    return pluginTranslations;
  }

  /**
   * 加载 JSON 文件
   */
  private async loadJsonFile(filePath: string): Promise<TranslationMap> {
    const raw = await readFile(filePath, 'utf8');
    return JSON.parse(raw) as TranslationMap;
  }
}
