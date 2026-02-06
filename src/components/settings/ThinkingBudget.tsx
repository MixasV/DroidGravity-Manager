import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { ThinkingBudgetConfig, ThinkingBudgetMode } from "../../types/config";

interface ThinkingBudgetProps {
    config: ThinkingBudgetConfig;
    onChange: (config: ThinkingBudgetConfig) => void;
}

const DEFAULT_CONFIG: ThinkingBudgetConfig = {
    mode: 'auto',
    custom_value: 24576,
};

export default function ThinkingBudget({
    config = DEFAULT_CONFIG,
    onChange,
}: ThinkingBudgetProps) {
    const { t } = useTranslation();

    // 使用本地 state 管理输入值，允许临时的无效输入
    const [inputValue, setInputValue] = useState(String(config.custom_value));

    // 同步外部 config 变化
    useEffect(() => {
        setInputValue(String(config.custom_value));
    }, [config.custom_value]);

    const handleModeChange = (mode: ThinkingBudgetMode) => {
        onChange({ ...config, mode });
    };

    // 输入时只更新本地 state
    const handleInputChange = (val: string) => {
        setInputValue(val);
    };

    // 失焦时校验并提交
    const handleInputBlur = () => {
        let num = parseInt(inputValue, 10);
        if (isNaN(num) || num < 1024) num = 1024;
        if (num > 65536) num = 65536;
        setInputValue(String(num));
        onChange({ ...config, custom_value: num });
    };

    const modes: ThinkingBudgetMode[] = ['auto', 'passthrough', 'custom'];

    return (
        <div className="space-y-4">
            {/* Header Section */}
            <div className="bg-blue-50/50 dark:bg-blue-900/10 border border-blue-100 dark:border-blue-800/30 rounded-lg p-4">
                <div className="space-y-1">
                    <h4 className="font-medium text-sm text-gray-900 dark:text-gray-100">
                        {t("settings.thinking_budget.title", { defaultValue: "思考预算 (Thinking Budget)" })}
                    </h4>
                    <p className="text-xs text-gray-500 dark:text-gray-400 leading-relaxed">
                        {t("settings.thinking_budget.description", {
                            defaultValue: "控制 AI 深度思考时的 Token 预算。Gemini 系列所有思考模型（包含 Pro 和 Flash）以及带 -thinking 后缀的模型受物理上限 24576 限制。",
                        })}
                    </p>
                </div>
            </div>

            {/* Mode Selector */}
            <div className="space-y-3">
                <label className="text-sm font-bold text-gray-700 dark:text-gray-300">
                    {t("settings.thinking_budget.mode_label", { defaultValue: "处理模式" })}
                </label>
                <div className="grid grid-cols-3 gap-3">
                    {modes.map((key) => (
                        <button
                            key={key}
                            onClick={() => handleModeChange(key)}
                            className={`p-4 rounded-xl border-2 transition-all ${config.mode === key
                                ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 shadow-md'
                                : 'border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-700 hover:bg-gray-50 dark:hover:bg-gray-800'
                                }`}
                        >
                            <div className="flex flex-col items-center gap-2">
                                <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                                    {t(`settings.thinking_budget.mode.${key}`)}
                                </span>
                                <span className="text-[10px] text-gray-500 dark:text-gray-400 text-center">
                                    {t(`settings.thinking_budget.mode.${key}_desc`)}
                                </span>
                            </div>
                        </button>
                    ))}
                </div>
            </div>

            {/* Mode-specific UI */}
            {config.mode === 'auto' && (
                <div className="bg-gray-50 dark:bg-gray-800/50 border border-gray-200 dark:border-gray-700 rounded-lg p-3">
                    <p className="text-xs text-gray-600 dark:text-gray-400">
                        {t("settings.thinking_budget.auto_hint", {
                            defaultValue: "自动模式：对 Gemini 协议模型（Pro/Flash/Thinking）以及启用 Web Search 的请求自动截断至 24576 以避免 API 错误。其他模型保持原始请求值。",
                        })}
                    </p>
                </div>
            )}

            {config.mode === 'passthrough' && (
                <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-700/30 rounded-lg p-3">
                    <p className="text-xs text-amber-700 dark:text-amber-400">
                        {t("settings.thinking_budget.passthrough_warning", {
                            defaultValue: "透传模式：直接使用调用方传入的 thinking_budget 值，不做任何限制。如果上游 API 不支持高值可能导致请求失败。",
                        })}
                    </p>
                </div>
            )}

            {config.mode === 'custom' && (
                <div className="space-y-3">
                    <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            {t("settings.thinking_budget.custom_value_label", { defaultValue: "自定义值" })}
                        </label>
                        <div className="flex items-center gap-3">
                            <input
                                type="number"
                                value={inputValue}
                                onChange={(e) => handleInputChange(e.target.value)}
                                onBlur={handleInputBlur}
                                className="w-40 bg-white dark:bg-base-100 border border-gray-200 dark:border-gray-700 rounded-lg px-4 py-2 text-sm font-mono focus:ring-2 focus:ring-blue-500/20 outline-none transition-all [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                                min={1024}
                                max={65536}
                                step={1024}
                            />
                            <span className="text-xs text-gray-500 dark:text-gray-400">
                                {t("settings.thinking_budget.tokens", { defaultValue: "tokens" })}
                            </span>
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-2">
                            {t("settings.thinking_budget.custom_value_hint", {
                                defaultValue: "范围：1024 - 65536。推荐值：24576（Flash 上限）或 51200（标准扩展思考）。",
                            })}
                        </p>
                    </div>
                    {config.custom_value > 24576 && (
                        <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-700/30 rounded-lg p-3">
                            <p className="text-xs text-amber-700 dark:text-amber-400">
                                {t("settings.thinking_budget.high_value_warning", {
                                    defaultValue: "当前值超过 24576，Gemini/Vertex AI 系列模型将由后端自动修正为此上限以防止请求失败。",
                                })}
                            </p>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}
