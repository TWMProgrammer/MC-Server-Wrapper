import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
    Calendar,
    Plus,
    Trash2,
    Clock,
    RefreshCw,
    Save,
    CheckCircle2,
    AlertCircle,
    AlertTriangle
} from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { ScheduledTask, ScheduleType } from './types'
import { useToast } from './hooks/useToast'
import { ConfirmDropdown } from './components/ConfirmDropdown'
import { Select } from './components/Select'

interface SchedulesTabProps {
    instanceId: string;
}

export function SchedulesTab({ instanceId }: SchedulesTabProps) {
    const [tasks, setTasks] = useState<ScheduledTask[]>([]);
    const [loading, setLoading] = useState(true);
    const [isAdding, setIsAdding] = useState(false);
    const { showToast } = useToast();
    const [newTask, setNewTask] = useState<{
        task_type: ScheduleType;
        cron: string;
    }>({
        task_type: 'Backup',
        cron: '0 0 * * *' // Default daily at midnight
    });

    const fetchTasks = async () => {
        try {
            const result = await invoke<ScheduledTask[]>('list_scheduled_tasks', { instanceId });
            setTasks(result);
        } catch (error) {
            console.error('Failed to fetch scheduled tasks:', error);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchTasks();
    }, [instanceId]);

    const handleAddTask = async () => {
        try {
            await invoke('add_scheduled_task', {
                instanceId,
                taskType: newTask.task_type,
                cron: newTask.cron
            });
            setIsAdding(false);
            fetchTasks();
        } catch (error) {
            console.error('Failed to add task:', error);
            showToast(`Failed to add task: ${error}`, 'error');
        }
    };

    const handleDeleteTask = async (taskId: string) => {
        try {
            await invoke('remove_scheduled_task', { instanceId, taskId });
            showToast('Schedule removed successfully');
            fetchTasks();
        } catch (error) {
            console.error('Failed to remove task:', error);
            showToast(`Failed to remove task: ${error}`, 'error');
        }
    };

    if (loading) {
        return (
            <div className="flex items-center justify-center h-full">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
            </div>
        );
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h2 className="text-2xl font-bold flex items-center gap-2">
                        <Calendar className="w-6 h-6 text-primary" />
                        Scheduled Tasks
                    </h2>
                    <p className="text-gray-400 mt-1">Manage automated backups and server restarts.</p>
                </div>
                <button
                    onClick={() => setIsAdding(true)}
                    className="flex items-center gap-2 px-4 py-2 bg-primary hover:bg-primary/90 text-white rounded-xl transition-all shadow-lg shadow-primary/20"
                >
                    <Plus className="w-4 h-4" />
                    Add Schedule
                </button>
            </div>

            <AnimatePresence>
                {isAdding && (
                    <motion.div
                        initial={{ opacity: 0, y: -20 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -20 }}
                        className="p-6 bg-surface border border-white/5 rounded-2xl space-y-4"
                    >
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div className="space-y-2">
                                <label className="text-sm font-medium text-gray-400">Task Type</label>
                                <Select
                                    value={newTask.task_type}
                                    onChange={(value) => setNewTask({ ...newTask, task_type: value as ScheduleType })}
                                    options={[
                                        { value: 'Backup', label: 'Backup' },
                                        { value: 'Restart', label: 'Restart' }
                                    ]}
                                />
                            </div>
                            <div className="space-y-2">
                                <label className="text-sm font-medium text-gray-400">Cron Expression</label>
                                <input
                                    type="text"
                                    value={newTask.cron}
                                    onChange={(e) => setNewTask({ ...newTask, cron: e.target.value })}
                                    placeholder="0 0 * * *"
                                    className="w-full bg-black/20 border border-white/10 rounded-xl px-4 py-2 focus:outline-none focus:border-primary transition-colors font-mono"
                                />
                                <p className="text-xs text-gray-500">
                                    Standard cron format: minute hour day month day-of-week
                                </p>
                            </div>
                        </div>
                        <div className="flex justify-end gap-3">
                            <button
                                onClick={() => setIsAdding(false)}
                                className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleAddTask}
                                className="flex items-center gap-2 px-6 py-2 bg-primary text-white rounded-xl hover:bg-primary/90 transition-all"
                            >
                                <Save className="w-4 h-4" />
                                Save Schedule
                            </button>
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>

            <div className="grid grid-cols-1 gap-4">
                {tasks.length === 0 ? (
                    <div className="text-center py-20 bg-surface/50 border border-white/5 rounded-2xl">
                        <Clock className="w-12 h-12 text-gray-600 mx-auto mb-4" />
                        <h3 className="text-lg font-medium text-gray-300">No schedules set</h3>
                        <p className="text-gray-500 mt-2">Automate your server maintenance with custom schedules.</p>
                    </div>
                ) : (
                    tasks.map((task) => (
                        <div
                            key={task.id}
                            className="group p-4 bg-surface border border-white/5 rounded-2xl flex items-center justify-between hover:border-primary/30 transition-all"
                        >
                            <div className="flex items-center gap-4 min-w-0 flex-1">
                                <div className={`p-3 rounded-xl shrink-0 ${task.task_type === 'Backup' ? 'bg-blue-500/10 text-blue-500' : 'bg-orange-500/10 text-orange-500'
                                    }`}>
                                    {task.task_type === 'Backup' ? <Save className="w-5 h-5" /> : <RefreshCw className="w-5 h-5" />}
                                </div>
                                <div className="min-w-0">
                                    <h4 className="font-bold text-lg truncate">{task.task_type}</h4>
                                    <div className="flex items-center gap-3 mt-1 text-sm">
                                        <span className="flex items-center gap-1 text-gray-400 shrink-0">
                                            <Clock className="w-3.5 h-3.5" />
                                            {task.cron}
                                        </span>
                                        {task.last_run && (
                                            <span className="flex items-center gap-1 text-gray-500 truncate">
                                                <CheckCircle2 className="w-3.5 h-3.5 text-green-500/70 shrink-0" />
                                                <span className="truncate">Last run: {new Date(task.last_run).toLocaleString()}</span>
                                            </span>
                                        )}
                                    </div>
                                </div>
                            </div>

                            <div className="shrink-0 ml-4">
                                <ConfirmDropdown
                                    onConfirm={() => handleDeleteTask(task.id)}
                                    title="Delete Schedule"
                                    message={`Are you sure you want to delete this ${task.task_type.toLowerCase()} schedule?`}
                                    variant="danger"
                                >
                                    <button
                                        className="p-2 text-gray-500 hover:text-red-500 hover:bg-red-500/10 rounded-lg transition-all opacity-0 group-hover:opacity-100"
                                    >
                                        <Trash2 className="w-5 h-5" />
                                    </button>
                                </ConfirmDropdown>
                            </div>
                        </div>
                    ))
                )}
            </div>

            <div className="p-4 bg-primary/5 border border-primary/10 rounded-2xl flex gap-4">
                <AlertCircle className="w-6 h-6 text-primary shrink-0" />
                <div className="text-sm text-gray-400">
                    <p className="font-medium text-gray-300 mb-1">Cron Expression Guide:</p>
                    <ul className="list-disc list-inside space-y-1">
                        <li><code className="text-primary">0 0 * * *</code> - Every day at midnight</li>
                        <li><code className="text-primary">0 */6 * * *</code> - Every 6 hours</li>
                        <li><code className="text-primary">*/30 * * * *</code> - Every 30 minutes</li>
                        <li><code className="text-primary">0 12 * * 1</code> - Every Monday at noon</li>
                    </ul>
                </div>
            </div>
        </div>
    );
}
