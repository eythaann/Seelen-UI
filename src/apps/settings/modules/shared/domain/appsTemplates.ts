import { AppTemplate } from '../../../../../shared.interfaces';

export interface AppTemplateDeclaration extends Omit<AppTemplate, 'apps'> {
  /** path from apps_templates */
  path: string;
}

export const AppsTemplates: AppTemplateDeclaration[] = [
  {
    name: 'Uncategorized',
    description: 'This are apps migrated from https://github.com/LGUG2Z/komorebi-application-specific-configuration that are still uncategorized',
    path: 'uncategorized.yml',
  },
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
  {
    name: 'Adobe',
    description: 'Settings for adobe apps.',
    path: 'adobe.yml',
  },
  {
    name: 'Video and Streaming',
    description: 'Settings for apps like OBS.',
    path: 'video-and-streaming.yml',
  },
  {
    name: 'Development',
    description: 'General settings devs 💻.',
    path: 'development.yml',
  },
  {
    name: 'Javascript Development',
    description: 'Settings devs focus on Javascript enviroment.',
    path: 'js-development.yml',
  },
];