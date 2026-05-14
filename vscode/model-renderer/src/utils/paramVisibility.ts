/**
 * Pure utility for determining whether a parameter should be rendered
 * given the current display settings. Shared by both tree and graph views.
 */
import { paramKey } from "./instancePath"

/** Returns true if the parameter should be rendered given current filters. */
export function isParamVisible(
    name: string,
    printLevel: string,
    instancePath: string[],
    showTrace: boolean,
    hideUnused: boolean,
    usedParamKeys: Set<string>,
): boolean {
    if (!showTrace && printLevel === "trace") return false
    if (hideUnused) {
        const key = paramKey(instancePath, name)
        if (!usedParamKeys.has(key)) return false
    }
    return true
}
