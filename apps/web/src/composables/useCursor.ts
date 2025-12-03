import { ref, onMounted, onUnmounted } from "vue";

const mouseX = ref(0);
const mouseY = ref(0);
const cursorActive = ref(false);

export function useCursor() {
  const handleMouseMove = (e: MouseEvent) => {
    mouseX.value = e.clientX;
    mouseY.value = e.clientY;
  };

  const handleMouseEnter = () => {
    cursorActive.value = true;
  };

  const handleMouseLeave = () => {
    cursorActive.value = false;
  };

  const initCursor = () => {
    onMounted(() => {
      window.addEventListener("mousemove", handleMouseMove);
    });

    onUnmounted(() => {
      window.removeEventListener("mousemove", handleMouseMove);
    });
  };

  return {
    mouseX,
    mouseY,
    cursorActive,
    handleMouseEnter,
    handleMouseLeave,
    initCursor,
  };
}
