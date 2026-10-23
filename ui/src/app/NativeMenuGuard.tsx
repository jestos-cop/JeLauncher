import { useEffect } from 'react'

/** Whether the webview's own menu is still useful here: in text fields and on selected text. */
function allowsNativeMenu(target: EventTarget | null) {
  if (target instanceof HTMLElement && target.closest('input, textarea, [contenteditable="true"]'))
    return true
  return (window.getSelection()?.toString() ?? '') !== ''
}

/** Keeps the webview menu (Reload, Inspect…) from opening on right click; the launcher's own menus
 * open where they apply. */
export function NativeMenuGuard() {
  useEffect(() => {
    const onContextMenu = (event: MouseEvent) => {
      if (!allowsNativeMenu(event.target)) event.preventDefault()
    }
    document.addEventListener('contextmenu', onContextMenu)
    return () => document.removeEventListener('contextmenu', onContextMenu)
  }, [])
  return null
}
