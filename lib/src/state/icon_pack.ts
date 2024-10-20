import { ResourceMetadata } from '.';

export class IconPack {
  info: ResourceMetadata = new ResourceMetadata();
  apps: Record<string, string> = {};
}
