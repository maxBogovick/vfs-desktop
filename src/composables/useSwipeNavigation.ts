import { onMounted, onUnmounted } from 'vue';

interface SwipeNavigationOptions {
  onSwipeLeft: () => void;
  onSwipeRight: () => void;
  canSwipeLeft: () => boolean;
  canSwipeRight: () => boolean;
}

export function useSwipeNavigation(options: SwipeNavigationOptions) {
  let accumulatedDeltaX = 0;
  let lastWheelTime = 0;
  let isSwipeInProgress = false;
  let wheelEventCount = 0;

  const SWIPE_THRESHOLD = 300; // Минимальное смещение для срабатывания (уменьшено)
  const RESET_DELAY = 200; // Время сброса накопленного значения (мс)
  const COOLDOWN_DELAY = 1000; // Задержка между свайпами (увеличено)
  const MIN_DELTA_X = 3; // Минимальный deltaX для учёта события

  const handleWheel = (event: WheelEvent) => {
    const now = Date.now();

    // Игнорируем слишком маленькие движения
    if (Math.abs(event.deltaX) < MIN_DELTA_X) {
      return;
    }

    // Проверяем, что это горизонтальный свайп (trackpad)
    // Игнорируем вертикальный скролл
    if (Math.abs(event.deltaX) <= Math.abs(event.deltaY)) {
      return;
    }

    // Если свайп уже выполняется, блокируем все события
    if (isSwipeInProgress) {
      event.preventDefault();
      return;
    }

    // Сбрасываем накопленное значение, если прошло много времени
    if (now - lastWheelTime > RESET_DELAY) {
      accumulatedDeltaX = 0;
      wheelEventCount = 0;
    }

    lastWheelTime = now;
    wheelEventCount++;

    // Накапливаем deltaX
    accumulatedDeltaX += event.deltaX;

    // Свайп вправо (назад) - отрицательный deltaX
    if (accumulatedDeltaX < -SWIPE_THRESHOLD && options.canSwipeLeft()) {
      event.preventDefault();
      event.stopPropagation();

      isSwipeInProgress = true;
      accumulatedDeltaX = 0;
      wheelEventCount = 0;

      options.onSwipeLeft();

      // Cooldown перед следующим свайпом
      setTimeout(() => {
        isSwipeInProgress = false;
      }, COOLDOWN_DELAY);
    }
    // Свайп влево (вперёд) - положительный deltaX
    else if (accumulatedDeltaX > SWIPE_THRESHOLD && options.canSwipeRight()) {
      event.preventDefault();
      event.stopPropagation();

      isSwipeInProgress = true;
      accumulatedDeltaX = 0;
      wheelEventCount = 0;

      options.onSwipeRight();

      // Cooldown перед следующим свайпом
      setTimeout(() => {
        isSwipeInProgress = false;
      }, COOLDOWN_DELAY);
    }
  };

  onMounted(() => {
    // Добавляем обработчик на document для глобального свайпа
    document.addEventListener('wheel', handleWheel, { passive: false });
  });

  onUnmounted(() => {
    document.removeEventListener('wheel', handleWheel);
  });

  return {
    // Можно добавить методы или состояния при необходимости
  };
}
