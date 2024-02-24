import { AppTemplate } from '../shared.interfaces';

export interface AppTemplateDeclaration extends Omit<AppTemplate, 'apps'> {
  /** path from apps_templates */
  path: string;
}

export const AppsTemplates: AppTemplateDeclaration[] = [
  {
    name: 'Core',
    description: 'The most low basic settings to work fine.',
    path: 'core.yml',
  },
  {
    name: 'Gaming related Apps',
    description: 'Settings for gamers 🎮.',
    path: 'gaming.yml',
  },
];