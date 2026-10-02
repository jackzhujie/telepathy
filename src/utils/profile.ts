export interface UserContext {
  name?: string;
  occupation?: string;
  industry?: string;
  interests: string[];
  language: 'auto' | 'zh' | 'en';
}

export function buildSystemPrompt(
  basePrompt: string,
  userContext: UserContext
): string {
  const parts = [];

  if (userContext.name || userContext.occupation || userContext.industry) {
    const info = [];
    if (userContext.name) info.push(`姓名：${userContext.name}`);
    if (userContext.occupation) info.push(`职业：${userContext.occupation}`);
    if (userContext.industry) info.push(`行业：${userContext.industry}`);
    parts.push(`【用户背景】${info.join('，')}`);
  }

  if (userContext.interests.length > 0) {
    parts.push(`【用户兴趣】${userContext.interests.join('、')}`);
  }

  let prompt = basePrompt;
  if (parts.length > 0) {
    prompt += '\n\n' + parts.join('\n');
  }

  if (userContext.language !== 'auto') {
    const langInstruction = userContext.language === 'zh'
      ? '\n\n【语言要求】请用中文回答所有问题。'
      : '\n\n【Language Requirement】Please answer all questions in English.';
    prompt += langInstruction;
  }

  return prompt;
}

export function detectQueryLanguage(query: string): 'zh' | 'en' {
  const chineseChars = query.match(/[\u4e00-\u9fa5]/g)?.length || 0;
  const totalChars = query.length;
  const chineseRatio = chineseChars / totalChars;

  return chineseRatio > 0.3 ? 'zh' : 'en';
}