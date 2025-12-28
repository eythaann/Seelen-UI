import { useInterval } from "libs/ui/react/utils/hooks";
import { cx } from "libs/ui/react/utils/styling";
import { Button, Skeleton } from "antd";
import { useAnimate } from "framer-motion";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import cs from "./News.module.css";

const BASE_NEWS_URL = "https://raw.githubusercontent.com/Seelen-Inc/slu-blog/refs/heads/main/news";

async function getNewNames(): Promise<string[]> {
  let response = await fetch(BASE_NEWS_URL + "/show_on_app.json");

  if (response.ok) {
    let data = await response.json();
    return data;
  }

  return [];
}

interface New {
  title: string;
  message: string;
  url: string;
  image: string;
}

async function getNew(name: string): Promise<New | null> {
  try {
    let response = await fetch(`${BASE_NEWS_URL}/${name}/metadata.json`);
    if (response.ok) {
      let data: New = await response.json();
      data.image = `${BASE_NEWS_URL}/${name}/image.png`;
      return data;
    }
  } catch (_error) {
    return null;
  }
  return null;
}

export function NoticeSlider() {
  const [news, setNews] = useState<New[]>([]);
  const [currentIdx, setCurrentIdx] = useState<number>(0);

  const [scope, animate] = useAnimate<HTMLDivElement>();
  const { t } = useTranslation();

  useEffect(() => {
    async function fetchData() {
      let names = await getNewNames();
      let news = await Promise.all(names.map((name) => getNew(name)));
      setNews(news.filter((item) => item !== null) as New[]);
    }
    fetchData();
  }, []);

  useInterval(
    () => {
      animate(scope.current, { opacity: 0 }).then(() => {
        setCurrentIdx((v) => v + 1);
        animate(scope.current, { opacity: 1 });
      });
    },
    10000,
    [currentIdx],
  );

  let current = news[currentIdx % news.length];

  return (
    <div className={cs.notices}>
      <div ref={scope} className={cs.notice}>
        {current
          ? (
            <>
              <img
                className={cs.image}
                src={current.image}
                alt={current.title}
              />
              <div className={cs.content}>
                <h3 className={cs.title}>{current.title}</h3>
                <p className={cs.message}>{current.message}</p>
                <div className={cs.linkButton}>
                  <Button href={current.url} target="_blank" type="primary">
                    {t("see_more")}
                  </Button>
                </div>
              </div>
            </>
          )
          : (
            <>
              <div className={cs.image} />
              <div className={cs.content}>
                <Skeleton active className={cs.title} paragraph={false} />
                <Skeleton active className={cs.message} title={false} />
                <div className={cs.linkButton}>
                  <Skeleton.Button active />
                </div>
              </div>
            </>
          )}
      </div>
      <div className={cs.pagination}>
        {news.map((_item, index) => (
          <div
            key={index}
            className={cx(cs.paginationDot, {
              [cs.active!]: index === currentIdx % news.length,
            })}
            onClick={() => {
              animate(scope.current, { opacity: 0 }).then(() => {
                setCurrentIdx(index);
                animate(scope.current, { opacity: 1 });
              });
            }}
          />
        ))}
      </div>
    </div>
  );
}
