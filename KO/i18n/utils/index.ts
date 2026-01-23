/**
 * 国际化工具函数
 */

import { i18n } from '../core/I18nManager.js';

/**
 * 创建翻译函数
 */
export function createTranslator(namespace?: string) {
  return (key: string, params?: Record<string, any>, locale?: string) => {
    const fullKey = namespace ? `${namespace}.${key}` : key;
    return i18n.t(fullKey, params, locale);
  };
}

/**
 * 格式化消息
 */
export function formatMessage(template: string, params: Record<string, any>): string {
  return template.replace(/\{\{(\w+)\}\}/g, (match, key) => {
    return params[key] !== undefined ? String(params[key]) : match;
  });
}

/**
 * 获取语言显示名称
 */
export function getLanguageDisplayName(locale: string, displayLocale?: string): string {
  const displayNames: Record<string, Record<string, string>> = {
    'zh-CN': {
      'en': 'Chinese (Simplified)',
      'zh-CN': '简体中文'
    },
    'en': {
      'en': 'English',
      'zh-CN': 'Chinese (Simplified)'
    }
  };
  
  const targetLocale = displayLocale || i18n.getCurrentLocale();
  return displayNames[locale]?.[targetLocale] || locale;
}

/**
 * 检查是否支持指定语言
 */
export function isLocaleSupported(locale: string): boolean {
  return i18n.getAvailableLocales().includes(locale);
}

/**
 * 获取最佳匹配语言
 */
export function getBestLocale(preferredLocales: string[]): string {
  const availableLocales = i18n.getAvailableLocales();
  const fallbackLocale = 'en';
  
  for (const preferred of preferredLocales) {
    if (availableLocales.includes(preferred)) {
      return preferred;
    }
    
    // 检查语言代码前缀匹配
    const languagePrefix = preferred.split('-')[0];
    const match = availableLocales.find(locale => 
      locale.startsWith(languagePrefix + '-')
    );
    if (match) {
      return match;
    }
  }
  
  return fallbackLocale;
}

/**
 * 翻译错误消息
 */
export function translateError(error: any, locale?: string): string {
  const errorKey = `errors.${error.code}`;
  const translated = i18n.t(errorKey, { message: error.message }, locale);
  
  // 如果翻译结果和 key 相同，说明没有找到翻译
  if (translated === errorKey) {
    return error.message || String(error);
  }
  
  return translated;
}

/**
 * 创建本地化字符串验证器
 */
export function createLocalizedValidator(locale?: string) {
  const targetLocale = locale || i18n.getCurrentLocale();
  
  return {
    required: (value: any) => {
      if (!value) {
        return i18n.t('validation.required', {}, targetLocale);
      }
      return null;
    },
    
    email: (value: string) => {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(value)) {
        return i18n.t('validation.email', {}, targetLocale);
      }
      return null;
    },
    
    minLength: (value: string, min: number) => {
      if (value.length < min) {
        return i18n.t('validation.minLength', { min }, targetLocale);
      }
      return null;
    },
    
    maxLength: (value: string, max: number) => {
      if (value.length > max) {
        return i18n.t('validation.maxLength', { max }, targetLocale);
      }
      return null;
    }
  };
}

/**
 * 本地化日期格式化
 */
export function formatDate(date: Date, locale?: string): string {
  const targetLocale = locale || i18n.getCurrentLocale();
  
  return new Intl.DateTimeFormat(targetLocale, {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  }).format(date);
}

/**
 * 本地化数字格式化
 */
export function formatNumber(number: number, locale?: string): string {
  const targetLocale = locale || i18n.getCurrentLocale();
  
  return new Intl.NumberFormat(targetLocale).format(number);
}
