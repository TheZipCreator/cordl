#pragma once

#include "concepts.hpp"

namespace {
namespace cordl_internals {
    /// @brief type to wrap a pointer to a T, not recommended to be used with anything that's not il2cpp compatible
    /// @tparam T type that instance points to
    template<typename T>
    requires(!il2cpp_reference_type<T>)
    struct Ptr {
        constexpr explicit Ptr(void* i) : instance(i) {}
        constexpr void* convert() const { return const_cast<void*>(instance); }

        constexpr Ptr(T* i) : instance(i) {}
        constexpr Ptr(T& i) : instance(&i) {}

        constexpr operator T&() const { return *static_cast<T*>(const_cast<void*>(instance)); }
        constexpr operator T*() const { return static_cast<T*>(const_cast<void*>(instance)); }
        T* operator ->() const { return static_cast<T*>(const_cast<void*>(instance)); }

        protected:
            void* instance;
    };

    // specific instantiation for void pointers
    template<>
    struct Ptr<void> {
        constexpr Ptr(void* i) : instance(i) {}
        constexpr void* convert() const { return const_cast<void*>(instance); }
        constexpr operator void*() const { return const_cast<void*>(instance); }

        protected:
            void* instance;
    };

    static_assert(sizeof(Ptr<void>) == sizeof(void*));

    // Ptr is neither Ref nor Val type
    template<> struct ::cordl_internals::GenRefTypeTrait<Ptr> { constexpr static bool value = false; };
    template<> struct ::cordl_internals::GenValueTypeTrait<Ptr> { constexpr static bool value = false; };
}
} // end anonymous namespace


template<typename T>
struct ::il2cpp_utils::il2cpp_type_check::il2cpp_no_arg_type<::cordl_internals::Ptr<T>> {
    static inline const Il2CppType* get() {
        static auto* typ = &::il2cpp_utils::il2cpp_type_check::il2cpp_no_arg_class<T>::get()->this_arg;
        return typ;
    }
};

template<typename T>
struct ::il2cpp_utils::il2cpp_type_check::il2cpp_arg_type<::cordl_internals::Ptr<T>> {
    static inline const Il2CppType* get([[maybe_unused]] ::cordl_internals::Ptr<T> arg) {
        return ::il2cpp_utils::il2cpp_type_check::il2cpp_no_arg_type<::cordl_internals::Ptr<T>>::get();
    }
};
