# Временная реализация волновой поверхности
Временная реализация волновой поверхности $\zeta_{w}(t)$, м,  двумерного нерегулярного волнения заданного спектра $S_v(\sigma)$, $м^2c$, определяется как суперпозиция N гармонических косинусоидальных волн по формуле:
> $$ \zeta_{w}(t)=\sum_1^N r_{w_i} cos(σ_i t + \phi_i)$$

Фаза $i$-ой волны $\phi_i$ выбирается случайно из диапазона [0 2$\pi$].

Радиус орбиты на свободной поверхности воды $r_{w_i}$, м, определяется по формуле:
> $$r_{w_i}=\sqrt{2A_i}$$

Выбор для $i$-ой волны круговой частоты $σ_i$, рад/с, и расчет площади $A_i$, $м^2$, под графиком спектра $S_v(\sigma)$, соответствующей частоте $σ_i$, производится следующим образом:
1. Методом Неймана для генерации случайных чисел по известной плотности распределения выбирается $(N-1)$ случайных значений $x_i$, которые разбивают ось $\sigma$ на $N$ случайных интервалов:
    - выбирается пара случайных чисел $x_i=rand$[$\sigma_{min}$ $\sigma_{max}$], $y_i=rand[0$ $S_vmax]$;
    - при попадании точки с координатами $[x_i$ $y_i]$ в площадь под график $S_v(\sigma)$, значение $x_i$ принимается в качестве очередного значения случайной величины. В противном случае значения отбрасываются;
2. Для каждого интервала методом трапеций определяется площадь $A_i$ и координата центра тяжести площади $A_i$ по оси $\sigma$, которая принимается как частота $σ_i$.