import type { ReactNode } from "react";

import type { PluginItemDisplay } from "../core";

export type PluginIcon = ReactNode | string;

export interface PluginActionView {
    id: string,
    label_key: string,
    icon: PluginIcon | null,
}

export interface PluginTypeView {
    the_type: string,
    actions: PluginActionView[],
    icon: PluginIcon | null,
}

export interface PluginView {
    id: string,
    types: PluginTypeView[],
    labels: Record<string, string>,
}

export interface PluginRegistry {
    register(plugin: PluginView): void;
    has(plugin_id: string): boolean;
    actions_of(item: PluginItemDisplay): PluginActionView[];
    current_action(item: PluginItemDisplay, action_index: number): PluginActionView | null;
    current_action_label(item: PluginItemDisplay | undefined, action_index: number): string | null;
    type_icon(the_type: string): PluginIcon | null;
    actions_of_type(the_type: string): PluginActionView[];
    action_icon(action_id: string): PluginIcon | null;
}

const NO_ACTIONS: PluginActionView[] = [];

class Registry implements PluginRegistry {
    private plugins: PluginView[] = [];
    private registered_ids = new Set<string>();
    private labels: Record<string, string> = {};

    register(plugin: PluginView): void {
        if (this.registered_ids.has(plugin.id)) return;
        this.registered_ids.add(plugin.id);
        this.plugins.push(plugin);
        Object.assign(this.labels, plugin.labels);
    }

    has(plugin_id: string): boolean {
        return this.registered_ids.has(plugin_id);
    }

    actions_of_type(the_type: string): PluginActionView[] {
        for (const plugin of this.plugins) {
            const type = plugin.types.find(t => t.the_type === the_type);
            if (type) return type.actions;
        }
        return NO_ACTIONS;
    }

    type_icon(the_type: string): PluginIcon | null {
        for (const plugin of this.plugins) {
            const type = plugin.types.find(t => t.the_type === the_type);
            if (type) return type.icon;
        }
        return null;
    }

    action_icon(action_id: string): PluginIcon | null {
        for (const plugin of this.plugins) {
            for (const type of plugin.types) {
                const action = type.actions.find(a => a.id === action_id);
                if (action) return action.icon;
            }
        }
        return null;
    }

    actions_of(item: PluginItemDisplay): PluginActionView[] {
        const actions = this.actions_of_type(item.the_type);
        return item.action_ids.map(id =>
            actions.find(action => action.id === id) ?? { id, label_key: id, icon: null });
    }

    current_action(item: PluginItemDisplay, action_index: number): PluginActionView | null {
        const actions = this.actions_of(item);
        return actions[Math.min(action_index, actions.length - 1)] ?? null;
    }

    current_action_label(item: PluginItemDisplay | undefined, action_index: number): string | null {
        if (!item) return null;
        const action = this.current_action(item, action_index);
        return action ? this.labels[action.label_key] ?? action.label_key : null;
    }
}

export const registry = new Registry();
