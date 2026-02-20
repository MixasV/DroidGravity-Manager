import { X, Clock, AlertCircle } from 'lucide-react';
import { createPortal } from 'react-dom';
import { Account, ModelQuota } from '../../types/account';
import { formatDate } from '../../utils/format';
import { useTranslation } from 'react-i18next';

interface AccountDetailsDialogProps {
    account: Account | null;
    onClose: () => void;
}

export default function AccountDetailsDialog({ account, onClose }: AccountDetailsDialogProps) {
    const { t } = useTranslation();
    
    if (!account) return null;

    // Check if this is a Kiro account
    const isKiro = account.provider === 'kiro';
    
    // For Kiro accounts, filter out credit info from models list
    const displayModels = isKiro 
        ? account.quota?.models?.filter(m => 
            !m.name.startsWith('kiro-') // Hide kiro-credits, kiro-monthly-limit, etc.
          ) || []
        : account.quota?.models || [];

    return createPortal(
        <div className="modal modal-open z-[100]">
            {/* Draggable Top Region */}
            <div data-tauri-drag-region className="fixed top-0 left-0 right-0 h-8 z-[110]" />

            <div className="modal-box relative max-w-3xl bg-white dark:bg-base-100 shadow-2xl rounded-2xl p-0 overflow-hidden">
                {/* Header */}
                <div className="px-6 py-5 border-b border-gray-100 dark:border-base-200 bg-gray-50/50 dark:bg-base-200/50 flex justify-between items-center">
                    <div className="flex items-center gap-3">
                        <h3 className="font-bold text-lg text-gray-900 dark:text-base-content">{t('accounts.details.title')}</h3>
                        <div className="px-2.5 py-0.5 rounded-full bg-gray-100 dark:bg-base-200 border border-gray-200 dark:border-base-300 text-xs font-mono text-gray-500 dark:text-gray-400">
                            {account.email}
                        </div>
                    </div>
                    <button
                        onClick={onClose}
                        className="btn btn-sm btn-circle btn-ghost text-gray-400 hover:bg-gray-100 dark:hover:bg-base-200 hover:text-gray-600 dark:hover:text-base-content transition-colors"
                    >
                        <X size={18} />
                    </button>
                </div>

                {/* Content */}
                <div className="p-6 space-y-4 max-h-[60vh] overflow-y-auto">
                    {/* Kiro Credits Summary */}
                    {isKiro && (() => {
                        const kiroCredits = account.quota?.models.find(m => m.name === 'kiro-credits');
                        const kiroMonthlyLimit = parseFloat(account.quota?.models.find(m => m.name === 'kiro-monthly-limit')?.reset_time || '0');
                        const kiroMonthlyUsed = parseFloat(account.quota?.models.find(m => m.name === 'kiro-monthly-used')?.reset_time || '0');
                        const kiroTrialLimit = parseFloat(account.quota?.models.find(m => m.name === 'kiro-trial-limit')?.reset_time || '0');
                        const kiroTrialUsed = parseFloat(account.quota?.models.find(m => m.name === 'kiro-trial-used')?.reset_time || '0');
                        const kiroTrialStatus = account.quota?.models.find(m => m.name === 'kiro-trial-status')?.reset_time || 'INACTIVE';
                        
                        return (
                            <div className="space-y-4 mb-6">
                                {/* Total Credits */}
                                <div className="p-4 rounded-xl border border-gray-100 dark:border-base-200 bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20">
                                    <div className="flex justify-between items-center mb-2">
                                        <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">Total Credits</span>
                                        <span className={`text-lg font-bold ${
                                            (kiroCredits?.percentage || 0) >= 50 ? 'text-emerald-600 dark:text-emerald-400' :
                                            (kiroCredits?.percentage || 0) >= 20 ? 'text-amber-600 dark:text-amber-400' : 
                                            'text-rose-600 dark:text-rose-400'
                                        }`}>
                                            {kiroCredits?.percentage || 0}%
                                        </span>
                                    </div>
                                    <div className="h-2 w-full bg-white/50 dark:bg-black/20 rounded-full overflow-hidden">
                                        <div
                                            className={`h-full rounded-full transition-all duration-500 ${
                                                (kiroCredits?.percentage || 0) >= 50 ? 'bg-emerald-500' :
                                                (kiroCredits?.percentage || 0) >= 20 ? 'bg-amber-400' :
                                                'bg-rose-500'
                                            }`}
                                            style={{ width: `${kiroCredits?.percentage || 0}%` }}
                                        />
                                    </div>
                                    {kiroCredits?.reset_time && (
                                        <div className="flex items-center gap-1.5 text-xs text-gray-500 dark:text-gray-400 mt-2">
                                            <Clock size={12} />
                                            <span>Resets: {formatDate(kiroCredits.reset_time)}</span>
                                        </div>
                                    )}
                                </div>
                                
                                {/* Monthly & Trial Credits */}
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    {/* Monthly Credits */}
                                    <div className="p-4 rounded-xl border border-blue-100 dark:border-blue-900/30 bg-blue-50/50 dark:bg-blue-900/10">
                                        <div className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Monthly Credits</div>
                                        <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                                            {(kiroMonthlyLimit - kiroMonthlyUsed).toFixed(1)}
                                        </div>
                                        <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                            of {kiroMonthlyLimit.toFixed(0)} total
                                        </div>
                                        <div className="text-xs text-gray-400 dark:text-gray-500 mt-2">
                                            Used: {kiroMonthlyUsed.toFixed(1)}
                                        </div>
                                    </div>
                                    
                                    {/* Free Trial Credits */}
                                    {kiroTrialStatus === 'ACTIVE' && kiroTrialLimit > 0 && (
                                        <div className="p-4 rounded-xl border border-emerald-100 dark:border-emerald-900/30 bg-emerald-50/50 dark:bg-emerald-900/10">
                                            <div className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Free Trial Credits</div>
                                            <div className="text-2xl font-bold text-emerald-600 dark:text-emerald-400">
                                                {(kiroTrialLimit - kiroTrialUsed).toFixed(1)}
                                            </div>
                                            <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                                of {kiroTrialLimit.toFixed(0)} total
                                            </div>
                                            <div className="text-xs text-gray-400 dark:text-gray-500 mt-2">
                                                Used: {kiroTrialUsed.toFixed(1)}
                                            </div>
                                        </div>
                                    )}
                                </div>
                            </div>
                        );
                    })()}
                    
                    {/* Models Grid (only for non-Kiro or if there are actual models) */}
                    {displayModels.length > 0 && (
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {displayModels.map((model: ModelQuota) => (
                        <div key={model.name} className="p-4 rounded-xl border border-gray-100 dark:border-base-200 bg-white dark:bg-base-100 hover:border-blue-100 dark:hover:border-blue-900 hover:shadow-sm transition-all group">
                            <div className="flex justify-between items-start mb-3">
                                <span className="text-sm font-medium text-gray-700 dark:text-gray-300 group-hover:text-blue-700 dark:group-hover:text-blue-400 transition-colors">
                                    {model.name}
                                </span>
                                <span
                                    className={`text-xs font-bold px-2 py-0.5 rounded-md ${model.percentage >= 50 ? 'bg-green-50 text-green-700 dark:bg-green-900/30 dark:text-green-400' :
                                        model.percentage >= 20 ? 'bg-orange-50 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400' :
                                            'bg-red-50 text-red-700 dark:bg-red-900/30 dark:text-red-400'
                                        }`}
                                >
                                    {model.percentage}%
                                </span>
                            </div>

                            {/* Progress Bar */}
                            <div className="h-1.5 w-full bg-gray-100 dark:bg-base-200 rounded-full overflow-hidden mb-3">
                                <div
                                    className={`h-full rounded-full transition-all duration-500 ${model.percentage >= 50 ? 'bg-emerald-500' :
                                        model.percentage >= 20 ? 'bg-orange-400' :
                                            'bg-red-500'
                                        }`}
                                    style={{ width: `${model.percentage}%` }}
                                ></div>
                            </div>

                            <div className="flex items-center gap-1.5 text-[10px] text-gray-400 dark:text-gray-500 font-mono">
                                <Clock size={10} />
                                <span>{t('accounts.reset_time')}: {formatDate(model.reset_time) || t('common.unknown')}</span>
                            </div>
                        </div>
                    ))}
                        </div>
                    )}
                    
                    {/* No Data Message (only if not Kiro and no models) */}
                    {!isKiro && displayModels.length === 0 && (
                        <div className="py-10 text-center text-gray-400 flex flex-col items-center">
                            <AlertCircle className="w-8 h-8 mb-2 opacity-20" />
                            <span>{t('accounts.no_data')}</span>
                        </div>
                    )}
                </div>
            </div>
            <div className="modal-backdrop bg-black/40 backdrop-blur-sm" onClick={onClose}></div>
        </div>,
        document.body
    );
}
