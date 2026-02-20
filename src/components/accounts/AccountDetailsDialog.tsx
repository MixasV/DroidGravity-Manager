import { X, Clock, AlertCircle, Globe } from 'lucide-react';
import { createPortal } from 'react-dom';
import { Account, ModelQuota } from '../../types/account';
import { formatDate } from '../../utils/format';
import { useTranslation } from 'react-i18next';
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { showToast } from '../common/ToastContainer';

interface AccountDetailsDialogProps {
    account: Account | null;
    onClose: () => void;
}

export default function AccountDetailsDialog({ account, onClose }: AccountDetailsDialogProps) {
    const { t } = useTranslation();
    const [proxyUrl, setProxyUrl] = useState(account?.individual_proxy || '');
    const [saving, setSaving] = useState(false);
    
    if (!account) return null;
    
    const handleSaveProxy = async () => {
        setSaving(true);
        try {
            await invoke('update_account_individual_proxy', {
                accountId: account.id,
                proxyUrl: proxyUrl || null
            });
            showToast(proxyUrl ? t('accounts.individual_proxy_saved') : t('accounts.individual_proxy_cleared'), 'success');
        } catch (error) {
            showToast(`Failed to save proxy: ${error}`, 'error');
        } finally {
            setSaving(false);
        }
    };

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
                    {/* Individual Proxy Section */}
                    <div className="p-4 rounded-xl border border-gray-100 dark:border-base-200 bg-gray-50/50 dark:bg-base-200/50">
                        <div className="flex items-center gap-2 mb-3">
                            <Globe className="w-4 h-4 text-indigo-600 dark:text-indigo-400" />
                            <h4 className="text-sm font-semibold text-gray-900 dark:text-base-content">
                                {t('accounts.individual_proxy')}
                            </h4>
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mb-3">
                            {t('accounts.individual_proxy_tooltip')}
                        </p>
                        <div className="flex gap-2">
                            <input
                                type="text"
                                className="input input-sm input-bordered flex-1 text-xs font-mono"
                                placeholder={t('accounts.individual_proxy_placeholder')}
                                value={proxyUrl}
                                onChange={(e) => setProxyUrl(e.target.value)}
                            />
                            <button
                                className="btn btn-sm btn-primary"
                                onClick={handleSaveProxy}
                                disabled={saving}
                            >
                                {saving ? t('common.loading') : t('accounts.individual_proxy_save')}
                            </button>
                            {proxyUrl && (
                                <button
                                    className="btn btn-sm btn-ghost"
                                    onClick={() => {
                                        setProxyUrl('');
                                        handleSaveProxy();
                                    }}
                                    disabled={saving}
                                >
                                    {t('accounts.individual_proxy_clear')}
                                </button>
                            )}
                        </div>
                    </div>
                    
                    {/* Models Grid */}
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {account.quota?.models?.map((model: ModelQuota) => (
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
                    )) || (
                            <div className="col-span-2 py-10 text-center text-gray-400 flex flex-col items-center">
                                <AlertCircle className="w-8 h-8 mb-2 opacity-20" />
                                <span>{t('accounts.no_data')}</span>
                            </div>
                        )}
                    </div>
                </div>
            </div>
            <div className="modal-backdrop bg-black/40 backdrop-blur-sm" onClick={onClose}></div>
        </div>,
        document.body
    );
}
