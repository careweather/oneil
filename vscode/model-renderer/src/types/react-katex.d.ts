declare module "react-katex" {
    import type { FC } from "react"

    interface MathProps {
        /** The LaTeX string to render. */
        math: string
        /** Override the block/inline rendering mode. */
        block?: boolean
        /** Error callback; called when KaTeX throws. */
        errorColor?: string
        renderError?: (error: Error) => React.ReactNode
    }

    export const InlineMath: FC<MathProps>
    export const BlockMath: FC<MathProps>
}
