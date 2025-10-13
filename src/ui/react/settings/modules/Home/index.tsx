import { RemoteResources } from "./MiniStore.tsx";
import { NoticeSlider } from "./News.tsx";

export function Home() {
  return (
    <>
      <NoticeSlider />
      <RemoteResources />
    </>
  );
}
