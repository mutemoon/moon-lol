import React, { useState, useEffect } from 'react'
import { Copy, Check, Play, Download, Code } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'

interface RonDialogProps {
  isOpen: boolean
  onClose: () => void
  ronText: string
  particleName: string
  onApplyRon: (newRon: string) => void
}

export const RonDialog: React.FC<RonDialogProps> = ({
  isOpen,
  onClose,
  ronText,
  particleName,
  onApplyRon,
}) => {
  const [editText, setEditText] = useState(ronText)
  const [copied, setCopied] = useState(false)

  useEffect(() => {
    setEditText(ronText)
  }, [ronText])

  const handleCopy = () => {
    navigator.clipboard.writeText(editText)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const handleDownload = () => {
    const blob = new Blob([editText], { type: 'text/plain;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `${particleName || 'particle'}.ron`
    link.click()
    URL.revokeObjectURL(url)
  }

  const handleApply = () => {
    onApplyRon(editText)
    onClose()
  }

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-3xl max-h-[85vh] flex flex-col bg-background/95 backdrop-blur-xl border-border/60 shadow-2xl">
        <DialogHeader className="flex flex-row items-center justify-between pb-2 border-b border-border/40">
          <div className="flex items-center gap-2">
            <Code className="w-5 h-5 text-purple-400" />
            <DialogTitle className="text-sm font-bold text-foreground">
              RON 粒子系统源码定义
            </DialogTitle>
          </div>
          <DialogDescription className="text-xs text-muted-foreground font-mono">
            {particleName}
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 my-2 min-h-0 flex flex-col">
          <textarea
            value={editText}
            onChange={(e) => setEditText(e.target.value)}
            spellCheck={false}
            className="w-full flex-1 min-h-[380px] p-3 rounded-md bg-muted/30 border border-border/40 text-xs font-mono text-foreground/90 resize-none focus:outline-none focus:ring-1 focus:ring-purple-500/50 leading-relaxed selection:bg-purple-500/30"
          />
        </div>

        <DialogFooter className="flex items-center justify-between pt-2 border-t border-border/40 sm:justify-between">
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={handleCopy}
              className="h-8 px-3 gap-1.5 text-xs border-border/60"
            >
              {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
              <span>{copied ? '已复制' : '复制到剪贴板'}</span>
            </Button>

            <Button
              size="sm"
              variant="outline"
              onClick={handleDownload}
              className="h-8 px-3 gap-1.5 text-xs border-border/60"
            >
              <Download className="w-3.5 h-3.5" />
              <span>下载 RON</span>
            </Button>
          </div>

          <div className="flex items-center gap-2">
            <Button size="sm" variant="ghost" onClick={onClose} className="h-8 text-xs">
              取消
            </Button>
            <Button
              size="sm"
              onClick={handleApply}
              className="h-8 px-4 gap-1.5 text-xs bg-purple-600 hover:bg-purple-500 text-white shadow-md shadow-purple-900/30"
            >
              <Play className="w-3.5 h-3.5 fill-current" />
              <span>应用并重播</span>
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
