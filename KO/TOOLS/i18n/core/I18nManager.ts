/**
 * 国际化管理器
 * 为 OpenCode 插件生态系统提供多语言支持
 */

export interface TranslationMap {
  [key: string]: string | TranslationMap;
}

export interface I18nConfig {
  defaultLocale: string;
  fallbackLocale: string;
  debug?: boolean;
}

export class I18nManager {
  private static instance: I18nManager;
  private currentLocale: string;
  private fallbackLocale: string;
  private translations: Map<string, TranslationMap> = new Map();
  private debug: boolean = false;

  private constructor(config: I18nConfig) {
    this.currentLocale = config.defaultLocale;
    this.fallbackLocale = config.fallbackLocale;
    this.debug = config.debug || false;
  }

  /**
   * 获取 I18nManager 单例
   */
  static getInstance(config?: I18nConfig): I18nManager {
    if (!I18nManager.instance) {
      if (!config) {
        throw new Error('I18nManager requires config on first initialization');
      }
      I18nManager.instance = new I18nManager(config);
    }
    return I18nManager.instance;
  }

  /**
   * 加载语言包
   */
  async loadLocale(locale: string, translations: TranslationMap): Promise<void> {
    this.translations.set(locale, translations);
    
    if (this.debug) {
      console.log(`[I18n] Loaded locale: ${locale}`);
    }
  }

  /**
   * 获取翻译文本
   */
  t(key: string, params?: Record<string, any>, locale?: string): string {
    const targetLocale = locale || this.currentLocale;
    const translation = this.getNestedKey(this.translations.get(targetLocale), key);
    
    if (translation !== undefined) {
      return this.interpolate(translation as string, params);
    }

    // 回退到默认语言
    if (targetLocale !== this.fallbackLocale) {
      const fallbackTranslation = this.getNestedKey(
        this.translations.get(this.fallbackLocale), 
        key
      );
      
      if (fallbackTranslation !== undefined) {
        if (this.debug) {
          console.log(`[I18n] Fallback for "${key}" in locale "${targetLocale}"`);
        }
        return this.interpolate(fallbackTranslation as string, params);
      }
    }

    // 如果都没有找到，返回 key 本身
    if (this.debug) {
      console.log(`[I18n] Missing translation for "${key}" in locale "${targetLocale}"`);
    }
    return key;
  }

  /**
   * 设置当前语言
   */
  setLocale(locale: string): void {
    if (!this.translations.has(locale)) {
      throw new Error(`Locale "${locale}" not loaded. Use loadLocale() first.`);
    }
    
    this.currentLocale = locale;
    
    if (this.debug) {
      console.log(`[I18n] Locale set to: ${locale}`);
    }
  }

  /**
   * 获取当前语言
   */
  getCurrentLocale(): string {
    return this.currentLocale;
  }

  /**
   * 获取可用语言列表
   */
  getAvailableLocales(): string[] {
    return Array.from(this.translations.keys());
  }

  /**
   * 从嵌套对象中获取键值
   */
  private getNestedKey(obj: any, key: string): any {
    return key.split('.').reduce((current, keyPart) => {
      return current?.[keyPart];
    }, obj);
  }

  /**
   * 插值处理
   */
  private interpolate(text: string, params?: Record<string, any>): string {
    if (!params) return text;
    
    return text.replace(/\{\{(\w+)\}\}/g, (match, key) => {
      return params[key] !== undefined ? String(params[key]) : match;
    });
  }

  /**
   * 重置管理器（主要用于测试）
   */
  reset(): void {
    this.translations.clear();
    this.currentLocale = 'en';
    this.fallbackLocale = 'en';
  }
}

/**
 * 全局 i18n 实例
 */
export const i18n = I18nManager.getInstance({
  defaultLocale: 'zh-CN',
  fallbackLocale: 'en',
  debug: false
});