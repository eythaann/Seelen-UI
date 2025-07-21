import { WallpaperId } from '@seelen-ui/lib/types';
import { ResourceText } from '@shared/components/ResourceText';
import { VerticalSortableSelect } from '@shared/components/SortableSelector';
import { useDispatch, useSelector } from 'react-redux';

import { newSelectors, RootActions } from '../shared/store/app/reducer';

export function WallpaperList() {
  const enabled = useSelector(newSelectors.wall.backgroundsV2);
  const wallpapers = useSelector(newSelectors.wallpapers);

  const d = useDispatch();

  function onChangeEnabled(backgroundsV2: WallpaperId[]) {
    d(RootActions.patchWall({ backgroundsV2 }));
  }

  return (
    <VerticalSortableSelect
      options={wallpapers.map((w) => ({
        value: w.id,
        label: <ResourceText text={w.metadata.displayName} />,
      }))}
      enabled={enabled}
      onChange={onChangeEnabled}
    />
  );
}
/*
export function _WallpaperList({ enabled, onChangeEnabled }: Props) {
  const wallpapers = useSelector(newSelectors.wallpapers);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 5,
      },
    }),
    useSensor(TouchSensor),
  );

  const { t } = useTranslation();

  const enabledList = wallpapers
    .filter((wallpaper) => enabled.includes(wallpaper.id))
    .toSorted((a, b) => enabled.indexOf(a.id) - enabled.indexOf(b.id));

  const disabledList = wallpapers.filter((wallpaper) => !enabled.includes(wallpaper.id));

  function handleDragEnd(e: DragEndEvent) {
    const { active, over } = e;
    if (!over || active.id === over.id) {
      return;
    }

    const oldPos = enabled.indexOf(active.id as WallpaperId);
    const newPos = enabled.indexOf(over.id as WallpaperId);
    const newEnabled = arrayMove(enabled, oldPos, newPos);
    onChangeEnabled(newEnabled);
  }

  return (
    <>
      <b>{t('wall.enabled')}</b>
      <ul className={cs.wallpaperList}>
        <DndContext onDragEnd={handleDragEnd} sensors={sensors}>
          <SortableContext items={enabled} strategy={verticalListSortingStrategy}>
            {enabledList.map((wallpaper) => (
              <Draggable key={wallpaper.id} id={wallpaper.id}>
                <ResourceText text={wallpaper.metadata.displayName} />
                <Switch
                  value={true}
                  onChange={() => {
                    onChangeEnabled(enabled.filter((id) => id !== wallpaper.id));
                  }}
                />
              </Draggable>
            ))}
          </SortableContext>
        </DndContext>
        {!enabledList.length && <div>{t('wall.no_background')}</div>}
      </ul>

      <b>{t('wall.available')}</b>
      <ul className={cs.wallpaperList}>
        {disabledList.map((wallpaper) => (
          <li key={wallpaper.id} className={cs.wallpaperEntry}>
            <ResourceText text={wallpaper.metadata.displayName} />
            <Switch
              value={false}
              onChange={() => {
                const dict = new Set(enabled);
                dict.add(wallpaper.id);
                onChangeEnabled(Array.from(dict));
              }}
            />
          </li>
        ))}
      </ul>
    </>
  );
}

function Draggable({ children, id }: { children: ComponentChildren; id: string }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id,
  });

  return (
    <li
      className={cs.wallpaperEntry}
      ref={setNodeRef}
      style={{
        transform: CSS.Translate.toString(transform),
        transition,
        opacity: isDragging ? 0.1 : 1,
      }}
    >
      <Icon iconName="GrDrag" {...(attributes as any)} {...listeners} />
      {children}
    </li>
  );
}
 */